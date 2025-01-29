use std::{path::PathBuf, time::SystemTimeError};

use quickemu_core::data::Arch;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigSearchError {
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
    #[error("An OS must be specified before searching for releases, editions, or architectures")]
    RequiredOS,
    #[error("A release is required before searching for editions")]
    RequiredRelease,
    #[error("An edition is required before selecting a config")]
    RequiredEdition,
    #[error("No OS matching {0} was found")]
    InvalidOS(String),
    #[error("No release {0} found for {1}")]
    InvalidRelease(String, String),
    #[error("No edition {0} found")]
    InvalidEdition(String),
    #[error("Architecture {0} not found including other parameters")]
    InvalidArchitecture(Arch),
    #[error("No editions are available for the specified release")]
    NoEditions,
}

#[derive(Error, Debug)]
pub enum DLError {
    #[error("A source does not currently exist for {0}")]
    UnsupportedSource(String),
    #[error("Invalid VM name {0}")]
    InvalidVMName(String),
    #[error("Unable to write to config file")]
    ConfigFileError(#[from] std::io::Error),
    #[error("Failed to serialize config data")]
    ConfigDataError(#[from] toml::ser::Error),
    #[error("File {0} was not successfully downloaded")]
    DownloadError(PathBuf),
    #[error("Invalid checksum {0}")]
    InvalidChecksum(String),
    #[error("Checksums did not match. Expected {0}, got {1}")]
    FailedValidation(String, String),
    #[error("VM Directory {0} already exists")]
    DirAlreadyExists(PathBuf),
}
