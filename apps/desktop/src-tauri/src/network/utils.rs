use std::{sync::LazyLock, time::Duration};

use log::debug;
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
        let query_string = serde_qs::to_string(q).map_err(|e| e.to_string())?;
        url.set_query(Some(&query_string));
        debug!("Generated query string: {}", query_string);
    }

    debug!("The url: {}", url);

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
