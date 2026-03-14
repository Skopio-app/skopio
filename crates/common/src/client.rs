use std::{path::PathBuf, sync::OnceLock};

use anyhow::anyhow;
use bytes::Bytes;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client,
};
use uuid::Uuid;

use crate::{error::CommonError, keyring::Keyring};

const SERVICE: &str = "skopio";
const ACCOUNT: &str = "bearer_token";
const DEV_BASE_URL: &str = "http://127.0.0.1:8080";
const PROD_BASE_URL: &str = "http://localhost";

static DEV_TRANSPORT: OnceLock<Result<Transport, String>> = OnceLock::new();
static PROD_TRANSPORT: OnceLock<Result<Transport, String>> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct Transport {
    client: Client,
    base: String,
}

impl Transport {
    pub fn new() -> Result<Self, CommonError> {
        let cached = if cfg!(debug_assertions) {
            DEV_TRANSPORT.get_or_init(init_dev_transport)
        } else {
            PROD_TRANSPORT.get_or_init(init_prod_transport)
        };

        cached.clone().map_err(|message| anyhow!(message).into())
    }

    pub async fn get(&self, path: &str) -> Result<Bytes, CommonError> {
        let url = self.url(path);
        let payload = self
            .client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(payload)
    }

    pub async fn post(&self, path: &str, body: Vec<u8>) -> Result<Bytes, CommonError> {
        let url = self.url(path);
        let payload = self
            .client
            .post(url)
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .send()
            .await?
            .error_for_status()?
            .bytes()
            .await?;
        Ok(payload)
    }
}

impl Transport {
    fn url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base, path)
        } else {
            format!("{}/{}", self.base, path)
        }
    }
}

fn init_dev_transport() -> Result<Transport, String> {
    build_transport(DEV_BASE_URL, "dev", None).map_err(|err| err.to_string())
}

fn init_prod_transport() -> Result<Transport, String> {
    let password = Uuid::new_v4().to_string();
    let token = Keyring::get_or_set_password(SERVICE, ACCOUNT, password.as_str())
        .map_err(|err| err.to_string())?;
    let sock = dirs::data_dir()
        .ok_or_else(|| anyhow!("Data dir not found"))
        .map_err(|err| err.to_string())?
        .join("com.samwahome.skopio/run/skopio.sock");

    build_transport(PROD_BASE_URL, &token, Some(sock)).map_err(|err| err.to_string())
}

fn build_transport(
    base: &str,
    token: &str,
    socket_path: Option<PathBuf>,
) -> Result<Transport, CommonError> {
    let mut default_headers = HeaderMap::new();
    let bearer = format!("Bearer {token}");
    let auth = HeaderValue::from_str(&bearer)
        .map_err(|err| anyhow!("Invalid bearer token header: {err}"))?;
    default_headers.insert(AUTHORIZATION, auth);

    let mut builder = Client::builder()
        .user_agent("skopio-client")
        .default_headers(default_headers);

    #[cfg(unix)]
    if let Some(sock) = socket_path {
        builder = builder.unix_socket(sock);
    }

    let client = builder.build()?;
    Ok(Transport {
        client,
        base: base.to_string(),
    })
}
