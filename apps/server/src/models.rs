use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ClientMessage {
    Duration(DurationRequest),
    Range(RangeRequest),
}
#[derive(Deserialize, Debug)]
pub struct DurationRequest {
    pub minutes: i64,
}

// TODO: Check whether to keep msg_type
#[derive(Debug, Deserialize)]
pub struct RangeRequest {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    pub msg_type: String,
    pub start_timestamp: String,
    pub end_timestamp: String,
}
