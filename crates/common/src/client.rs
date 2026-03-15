use std::{path::PathBuf, sync::OnceLock, time::Duration};

use anyhow::anyhow;
use bytes::Bytes;
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE},
    Client, RequestBuilder,
};
use serde::Serialize;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::{error::CommonError, keyring::Keyring};

const SERVICE: &str = "skopio";
const ACCOUNT: &str = "bearer_token";
const DEV_BASE_URL: &str = "http://127.0.0.1:8080";
const PROD_BASE_URL: &str = "http://localhost";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

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
        let request_path = Self::build_path(path);
        let request = self.client.get(self.url(&request_path));
        self.execute("GET", request_path, request).await
    }

    pub async fn post(&self, path: &str, body: Vec<u8>) -> Result<Bytes, CommonError> {
        let request_path = Self::build_path(path);
        let request = self
            .client
            .post(self.url(&request_path))
            .header(CONTENT_TYPE, "application/json")
            .body(body);
        self.execute("POST", request_path, request).await
    }
}

impl Transport {
    async fn execute(
        &self,
        method: &'static str,
        path: String,
        request: RequestBuilder,
    ) -> Result<Bytes, CommonError> {
        debug!(method, path = %path, base = %self.base, "sending transport request");

        let response = request.send().await.map_err(|error| {
            warn!(method, path = %path, error = %error, "transport request failed before response");
            error
        })?;
        let status = response.status();

        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            warn!(
                method,
                path = %path,
                status = %status,
                body = %body,
                "transport request returned non-success status"
            );
            return Err(anyhow!("HTTP {status} for {method} {path}: {body}").into());
        }

        let payload = response.bytes().await.map_err(|error| {
            warn!(method, path = %path, status = %status, error = %error, "failed to read transport response body");
            error
        })?;
        debug!(method, path = %path, status = %status, body_len = payload.len(), "transport request completed");
        Ok(payload)
    }

    fn url(&self, path: &str) -> String {
        if path.starts_with('/') {
            format!("{}{}", self.base, path)
        } else {
            format!("{}/{}", self.base, path)
        }
    }
}

impl Transport {
    pub fn build_path(path: &str) -> String {
        if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        }
    }

    pub fn build_path_with_query<T>(path: &str, query: Option<&T>) -> Result<String, CommonError>
    where
        T: Serialize,
    {
        let mut full_path = Self::build_path(path);
        if let Some(query) = query {
            let qs = serde_qs::to_string(query)?;
            if !qs.is_empty() {
                full_path.push('?');
                full_path.push_str(&qs);
            }
        }
        Ok(full_path)
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
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT)
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
