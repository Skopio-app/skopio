use std::{collections::HashMap, sync::LazyLock, time::Duration};

use reqwest::Client;
use serde::{Deserialize, Serialize};
use url::Url;

static HTTP_CLIENT: LazyLock<Client> = LazyLock::new(|| {
    Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to create request client")
});

const SERVER_URL: &str = "http://localhost:8080";

pub async fn req_json<TRes, TQuery>(path: &str, query: Option<&TQuery>) -> Result<TRes, String>
where
    TRes: for<'de> Deserialize<'de>,
    TQuery: Serialize,
{
    let mut url = Url::parse(&format!("{}/{}", SERVER_URL, path)).map_err(|e| e.to_string())?;

    if let Some(q) = query {
        let map = to_string_map(q)?;
        url.query_pairs_mut().extend_pairs(map);
    }

    let res = HTTP_CLIENT
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        res.json::<TRes>().await.map_err(|e| e.to_string())
    } else {
        Err(res
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string()))
    }
}

fn to_string_map<T: Serialize>(value: &T) -> Result<HashMap<String, String>, String> {
    let json = serde_json::to_value(value).map_err(|e| e.to_string())?;
    let obj = json.as_object().ok_or("Expected object for query")?;

    let mut map = HashMap::new();
    for (k, v) in obj {
        if !v.is_null() {
            map.insert(k.to_string(), v.to_string().trim_matches('"').to_string());
        }
    }

    Ok(map)
}
