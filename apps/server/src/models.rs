use serde::Deserialize;

#[derive(Deserialize)]
pub struct DurationRequest {
    pub minutes: i64,
}
