use std::{
    path::{Path, PathBuf},
    str,
};

use anyhow::anyhow;
use reqwest::Client;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::UnixStream,
};
use uuid::Uuid;

use crate::{error::CommonError, keyring::Keyring};

const SERVICE: &str = "skopio";
const ACCOUNT: &str = "bearer_token";

#[derive(Clone, Debug)]
pub enum Transport {
    DevTcp { base: String, token: String },
    ProdUds { sock: PathBuf, token: String },
}

impl Transport {
    pub fn new() -> Result<Self, CommonError> {
        if cfg!(debug_assertions) {
            Ok(Transport::DevTcp {
                base: "http://127.0.0.1:8080".into(),
                token: "dev".into(),
            })
        } else {
            let password = Uuid::new_v4().to_string();
            let token = Keyring::get_or_set_password(SERVICE, ACCOUNT, password.as_str())?;
            let run_dir = dirs::data_dir()
                .ok_or_else(|| anyhow!("Data dir not found"))?
                .join("com.samwahome.skopio/run");
            Ok(Transport::ProdUds {
                sock: run_dir.join("skopio.sock"),
                token,
            })
        }
    }

    pub async fn get(&self, path: &str) -> Result<String, CommonError> {
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

    pub async fn post_json(&self, path: &str, json: &str) -> Result<String, CommonError> {
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
) -> Result<String, CommonError> {
    let mut stream = UnixStream::connect(sock_path).await?;

    let body = body_opt.unwrap_or("");
    let has_body = !body.is_empty();
    let content_len = if has_body { body.len() } else { 0 };

    let mut req = format!(
        "{method} {path} HTTP/1.1\r\n\
         Host: localhost\r\n\
         Connection: close\r\n\
         User-Agent: skopio-client\r\n\
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
    stream.flush().await?;

    let mut buf = Vec::with_capacity(4096);
    let mut tmp = [0u8; 2048];
    let headers_end = loop {
        let n = stream.read(&mut tmp).await?;
        if n == 0 {
            return Err(anyhow!("unexpected EOF before headers").into());
        }
        buf.extend_from_slice(&tmp[..n]);
        if let Some(pos) = find_headers_end(&buf) {
            break pos;
        }
        if buf.len() > 64 * 1024 {
            return Err(anyhow!("headers too large").into());
        }
    };

    let (head, remain) = buf.split_at(headers_end);
    let head_str = std::str::from_utf8(head)?;
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

    let mut body_bytes = Vec::new();
    if !remain.is_empty() {
        body_bytes.extend_from_slice(remain);
    }

    if chunked {
        read_chunked(&mut stream, &mut body_bytes).await?;
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
        return Err(anyhow!(text).into());
    }
    Ok(text)
}

fn find_headers_end(buf: &[u8]) -> Option<usize> {
    buf.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|pos| pos + 4)
}

async fn read_chunked(stream: &mut UnixStream, out: &mut Vec<u8>) -> Result<(), CommonError> {
    let mut buf = Vec::new();
    let mut tmp = [0u8; 1024];

    loop {
        let line = read_line(stream, &mut buf, &mut tmp).await?;
        let size = usize::from_str_radix(String::from_utf8(line).unwrap_or_default().trim(), 16)?;
        if size == 0 {
            let _ = read_line(stream, &mut buf, &mut tmp).await.ok();
            break;
        }
        while buf.len() < size {
            let n = stream.read(&mut tmp).await?;
            if n == 0 {
                return Err(anyhow!("Unexpected EOF in chunked payload"))?;
            }
            buf.extend_from_slice(&tmp[..n]);
        }
        out.extend_from_slice(&buf[..size]);
        buf.drain(..size);
        while buf.len() < 2 {
            let n = stream.read(&mut tmp).await?;
            if n == 0 {
                return Err(anyhow!("Unexpected EOF after chunk"))?;
            }
            buf.extend_from_slice(&tmp[..n]);
        }
        if &buf[..2] != b"\r\n" {
            return Err(anyhow!("Missing CRLF after chunk"))?;
        }
        buf.drain(..2);
    }
    Ok(())
}

async fn read_line(
    stream: &mut UnixStream,
    buf: &mut Vec<u8>,
    tmp: &mut [u8; 1024],
) -> Result<Vec<u8>, CommonError> {
    loop {
        if let Some(pos) = buf.windows(2).position(|w| w == b"\r\n") {
            let line = buf.drain(..pos + 2).collect::<Vec<u8>>();
            return Ok(line[..line.len() - 2].to_vec());
        }
        let n = stream.read(tmp).await?;
        if n == 0 {
            return Err(anyhow!("Unexpected EOF in chunked body"))?;
        }
        buf.extend_from_slice(&tmp[..n]);
    }
}
