use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};

#[derive(Clone)]
pub struct AuthCfg {
    pub bearer: Arc<str>,
}

pub async fn bearer_auth(
    State(cfg): State<AuthCfg>,
    req: axum::http::Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let Some(hv) = req.headers().get(header::AUTHORIZATION) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    let Ok(val) = hv.to_str() else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if let Some(tok) = val.strip_prefix("Bearer ") {
        if tok == &*cfg.bearer {
            return Ok(next.run(req).await);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}
