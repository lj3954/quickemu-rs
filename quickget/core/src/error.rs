use std::{fmt, path::PathBuf, time::SystemTimeError};

use quickemu_core::data::Arch;

use crate::fl;

#[derive(derive_more::From, Debug)]
pub enum ConfigSearchError {
    FailedCacheDir,
    InvalidCacheDir(PathBuf),
    #[from]
    InvalidSystemTime(SystemTimeError),
    #[from]
    FailedCacheFile(std::io::Error),
    #[from]
    FailedDownload(reqwest::Error),
    #[from]
    FailedJson(serde_json::Error),
    RequiredOS,
    RequiredRelease,
    RequiredEdition,
    InvalidOS(String),
    InvalidRelease(String, String),
    InvalidEdition(String),
    InvalidArchitecture(Arch),
    NoEditions,
}

impl std::error::Error for ConfigSearchError {}
impl fmt::Display for ConfigSearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::FailedCacheDir => fl!("failed-cache-dir"),
            Self::InvalidCacheDir(dir) => fl!("invalid-cache-dir", dir = dir.display().to_string()),
            Self::InvalidSystemTime(err) => fl!("invalid-system-time", err = err.to_string()),
            Self::FailedCacheFile(err) => fl!("failed-cache-file", err = err.to_string()),
            Self::FailedDownload(err) => fl!("failed-download", err = err.to_string()),
            Self::FailedJson(err) => fl!("failed-json", err = err.to_string()),
            Self::RequiredOS => fl!("required-os"),
            Self::RequiredRelease => fl!("required-release"),
            Self::RequiredEdition => fl!("required-edition"),
            Self::InvalidOS(os) => fl!("invalid-os", os = os),
            Self::InvalidRelease(rel, os) => fl!("invalid-release", rel = rel, os = os),
            Self::InvalidEdition(edition) => fl!("invalid-edition", edition = edition),
            Self::InvalidArchitecture(arch) => fl!("invalid-arch", arch = arch.to_string()),
            Self::NoEditions => fl!("no-editions"),
        };
        f.write_str(&text)
    }
}

#[derive(derive_more::From, Debug)]
pub enum DLError {
    UnsupportedSource(String),
    InvalidVMName(String),
    #[from]
    ConfigFileError(std::io::Error),
    #[from]
    ConfigDataError(toml::ser::Error),
    DownloadError(PathBuf),
    InvalidChecksum(String),
    FailedValidation(String, String),
    DirAlreadyExists(PathBuf),
}

impl std::error::Error for DLError {}
impl fmt::Display for DLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::UnsupportedSource(os) => fl!("unsupported-source", os = os),
            Self::InvalidVMName(vm_name) => fl!("invalid-vm-name", vm_name = vm_name),
            Self::ConfigFileError(err) => fl!("config-file-error", err = err.to_string()),
            Self::ConfigDataError(err) => fl!("config-data-error", err = err.to_string()),
            Self::DownloadError(file) => fl!("download-error", file = file.display().to_string()),
            Self::InvalidChecksum(cs) => fl!("invalid-checksum", cs = cs),
            Self::FailedValidation(expected, actual) => fl!("failed-validation", expected = expected, actual = actual),
            Self::DirAlreadyExists(dir) => fl!("dir-exists", dir = dir.display().to_string()),
        };
        f.write_str(&text)
    }
}
