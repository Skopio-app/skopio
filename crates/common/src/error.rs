use std::{num::ParseIntError, str::Utf8Error};

use thiserror::Error;

/// Errors returned by the common crate
#[derive(Error, Debug)]
pub enum CommonError {
    /// reqwest errors
    #[error("Reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// Used for IO errors
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),

    /// Anyhow type errors
    #[error("Anyhow error: {0}")]
    Anyhow(#[from] anyhow::Error),

    /// Used for ParseInt errors
    #[error("ParseInt error: {0}")]
    ParseInt(#[from] ParseIntError),

    /// Used for keyring errors
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    /// Used for UTF8 errors
    #[error("UTF8 error: {0}")]
    Utf8(#[from] Utf8Error),
}
