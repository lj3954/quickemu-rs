use std::{path::PathBuf, time::SystemTimeError};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum QuickgetError {
    #[error("Failed to determine system cache directory")]
    FailedCacheDir,
    #[error("Cache directory {0} does not exist.")]
    InvalidCacheDir(PathBuf),
    #[error("Invalid system time")]
    InvalidSystemTime(#[from] SystemTimeError),
    #[error("Unable to interact with cache file")]
    FailedCacheFile(#[from] std::io::Error),
    #[error("Failed to download cache file")]
    FailedDownload(#[from] reqwest::Error),
    #[error("Could not serialize JSON data")]
    FailedJson(#[from] serde_json::Error),
}
