pub mod data;
pub mod events;
pub mod insights;
pub mod summaries;

use common::client::Transport;
use serde::{Deserialize, Serialize};

pub async fn req_json<TRes, TQuery>(path: &str, query: Option<&TQuery>) -> Result<TRes, String>
where
    TRes: for<'de> Deserialize<'de>,
    TQuery: Serialize,
{
    let mut full_path = String::new();
    if !path.starts_with('/') {
        full_path.push('/');
    }
    full_path.push_str(path);

    if let Some(q) = query {
        let qs = serde_qs::to_string(q).map_err(|e| e.to_string())?;
        if !qs.is_empty() {
            full_path.push('?');
            full_path.push_str(&qs);
        }
    }

    let transport = Transport::detect().map_err(|e| e.to_string())?;
    let text = transport.get(&full_path).await.map_err(|e| e.to_string())?;
    serde_json::from_str::<TRes>(&text).map_err(|e| e.to_string())
}

pub async fn post_json<TReq, TRes>(path: &str, body: &TReq) -> Result<TRes, String>
where
    TReq: Serialize + ?Sized,
    TRes: for<'de> Deserialize<'de>,
{
    let mut full_path = String::new();
    if !path.starts_with('/') {
        full_path.push('/');
    }
    full_path.push_str(path);

    let json = serde_json::to_string(body).map_err(|e| e.to_string())?;
    let transport = Transport::detect().map_err(|e| e.to_string())?;
    let text = transport
        .post_json(&full_path, &json)
        .await
        .map_err(|e| e.to_string())?;

    serde_json::from_str::<TRes>(&text).map_err(|e| e.to_string())
}
