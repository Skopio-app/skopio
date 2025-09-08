use std::{
    path::{Path, PathBuf},
    str,
};

use anyhow::{anyhow, Context, Result};
use reqwest::Client;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use uuid::Uuid;

use crate::keyring::Keyring;

const SERVICE: &str = "skopio";
const ACCOUNT: &str = "bearer_token";

#[derive(Clone)]
pub enum Transport {
    DevTcp { base: String, token: String },
    ProdUds { sock: PathBuf, token: String },
}

impl Transport {
    pub fn detect() -> Result<Self> {
        let password = Uuid::new_v4().to_string();

        if cfg!(debug_assertions) {
            Ok(Transport::DevTcp {
                base: "http://127.0.0.1:8080".into(),
                token: password,
            })
        } else {
            let token = Keyring::get_or_set_password(SERVICE, ACCOUNT, password.as_str())?;
            let run_dir = dirs::data_dir()
                .ok_or_else(|| anyhow!("Data dir not found"))?
                .join("com.samwahome.com/run");
            Ok(Transport::ProdUds {
                sock: run_dir.join("skopio.sock"),
                token,
            })
        }
    }

    pub async fn get(&self, path: &str) -> Result<String> {
        match self {
            Transport::DevTcp { base, token } => {
                let url = format!("{base}{path}");
                let txt = Client::new()
                    .get(&url)
                    .bearer_auth(token)
                    .send()
                    .await?
                    .error_for_status()?
                    .text()
                    .await?;
                Ok(txt)
            }
            Transport::ProdUds { sock, token } => uds_http(sock, "GET", path, token, None).await,
        }
    }

    pub async fn post_json(&self, path: &str, json: &str) -> Result<String> {
        match self {
            Transport::DevTcp { base, token } => {
                let url = format!("{base}{path}");
                let txt = reqwest::Client::new()
                    .post(&url)
                    .bearer_auth(token)
                    .header("content-type", "application/json")
                    .body(json.to_owned())
                    .send()
                    .await?
                    .error_for_status()?
                    .text()
                    .await?;
                Ok(txt)
            }
            Transport::ProdUds { sock, token } => {
                uds_http(sock, "POST", path, token, Some(json)).await
            }
        }
    }
}

async fn uds_http(
    sock_path: &Path,
    method: &str,
    path: &str,
    bearer: &str,
    body_opt: Option<&str>,
) -> Result<String> {
    let mut stream = UnixStream::connect(sock_path)
        .await
        .with_context(|| format!("connect {}", sock_path.display()))?;
    let body = body_opt.unwrap_or("");
    let has_body = !body.is_empty();
    let content_len = if has_body { body.len() } else { 0 };
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\n\
        Host: localhost\r\n\
        Connection: close\r\n\
        Authorization: Bearer {bearer}\r\n"
    );
    if has_body {
        req.push_str("Content-Type: application/json\r\n");
        req.push_str(&format!("Content-Length: {content_len}\r\n"));
    } else {
        req.push_str("Content-Length: 0\r\n");
    }
    req.push_str("\r\n");

    stream.write_all(req.as_bytes()).await?;
    if has_body {
        stream.write_all(body.as_bytes()).await?;
    }
    stream.shutdown().await.ok();

    // Read response
    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 1024];
    let headers_end = loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            break None.ok_or_else(|| anyhow!("unexpected EOF before headers"))?;
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break pos + 4;
        }
        if buf.len() > 64 * 1024 {
            return Err(anyhow!("headers too large"));
        }
    };

    let head = &buf[..headers_end];
    let head_str = str::from_utf8(head).map_err(|_| anyhow!("bad headers utf8"))?;
    let mut lines = head_str.split("\r\n");
    let status: u16 = lines
        .next()
        .ok_or_else(|| anyhow!("Missing status line"))?
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| anyhow!("Bad status line"))?
        .parse()?;

    let mut content_len: Option<usize> = None;
    let mut chunked = false;
    for line in lines {
        if line.is_empty() {
            break;
        }
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("content-length:") {
            content_len = v.trim().parse::<usize>().ok();
        } else if let Some(v) = lower.strip_prefix("transfer-encoding:") {
            chunked = v.trim().eq_ignore_ascii_case("chunked");
        }
    }

    // Read body
    let mut body_bytes = Vec::new();
    if chunked {
        read_chunked(&mut stream, &mut body_bytes).await?;
        if buf.len() > headers_end {
            body_bytes.extend_from_slice(&buf[headers_end..]);
        }
    } else if let Some(len) = content_len {
        while body_bytes.len() < len {
            let n = stream.read(&mut tmp).await?;
            if n == 0 {
                break;
            }
            body_bytes.extend_from_slice(&tmp[..n]);
        }
        body_bytes.truncate(len);
    } else {
        if buf.len() > headers_end {
            body_bytes.extend_from_slice(&buf[headers_end..]);
        }
        loop {
            let n = stream.read(&mut tmp).await?;
            if n == 0 {
                break;
            }
            body_bytes.extend_from_slice(&tmp[..n]);
        }
    }
    let text = String::from_utf8(body_bytes).unwrap_or_default();
    if !(200..300).contains(&status) {
        return Err(anyhow!(text));
    }
    Ok(text)
}

async fn read_chunked(stream: &mut UnixStream, out: &mut Vec<u8>) -> Result<()> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];

    loop {
        let line = read_line(stream, &mut buf, &mut tmp).await?;
        let size = usize::from_str_radix(String::from_utf8(line).unwrap_or_default().trim(), 16)
            .map_err(|_| anyhow!("Bad chunk size"))?;
        if size == 0 {
            let _ = read_line(stream, &mut buf, &mut tmp).await.ok();
            break;
        }
        while buf.len() < size {
            let n = stream.read(&mut tmp).await?;
            if n == 0 {
                return Err(anyhow!("Unexpected EOF in chunked payload"));
            }
            buf.extend_from_slice(&tmp[..n]);
        }
        out.extend_from_slice(&buf[..size]);
        buf.drain(..size);
        while buf.len() < 2 {
            let n = stream.read(&mut tmp).await?;
            if n == 0 {
                return Err(anyhow!("Unexpected EOF after chunk"));
            }
            buf.extend_from_slice(&tmp[..n]);
        }
        if &buf[..2] != b"\r\n" {
            return Err(anyhow!("Missing CRLF after chunk"));
        }
        buf.drain(..2);
    }
    Ok(())
}

async fn read_line(
    stream: &mut UnixStream,
    buf: &mut Vec<u8>,
    tmp: &mut [u8; 1024],
) -> Result<Vec<u8>> {
    loop {
        if let Some(pos) = buf.windows(2).position(|w| w == b"\r\n") {
            let line = buf.drain(..pos + 2).collect::<Vec<u8>>();
            return Ok(line[..line.len() - 2].to_vec());
        }
        let n = stream.read(tmp).await?;
        if n == 0 {
            return Err(anyhow!("Unexpected EOF in chunked body"));
        }
        buf.extend_from_slice(&tmp[..n]);
    }
}
