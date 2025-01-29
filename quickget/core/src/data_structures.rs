#![allow(dead_code)]
pub use quickemu_core::data::{Arch, BootType, DiskFormat, GuestOS};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OS {
    pub name: String,
    pub pretty_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub releases: Vec<Config>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub release: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    #[serde(flatten, default, skip_serializing_if = "is_default", deserialize_with = "default_if_empty")]
    pub guest_os: GuestOS,
    #[serde(flatten, default, skip_serializing_if = "is_default", deserialize_with = "default_if_empty")]
    pub arch: Arch,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub iso: Vec<Source>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub img: Vec<Source>,
    #[serde(default = "default_disk", skip_serializing_if = "is_default_disk")]
    pub disk_images: Option<Vec<Disk>>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub boot: BootType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tpm: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ram: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            release: "latest".to_string(),
            edition: None,
            guest_os: GuestOS::Linux,
            arch: Arch::default(),
            iso: Vec::new(),
            img: Vec::new(),
            disk_images: default_disk(),
            boot: Default::default(),
            tpm: None,
            ram: None,
        }
    }
}
fn is_default_disk(disk: &Option<Vec<Disk>>) -> bool {
    disk == &default_disk()
}
fn default_disk() -> Option<Vec<Disk>> {
    Some(vec![Default::default()])
}
fn is_default<T: Default + PartialEq>(input: &T) -> bool {
    input == &T::default()
}
pub fn default_if_empty<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de> + Default,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Disk {
    pub source: Source,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(default, flatten, skip_serializing_if = "is_default", deserialize_with = "default_if_empty")]
    pub format: DiskFormat,
}
impl Default for Disk {
    fn default() -> Self {
        Self {
            source: Source::FileName("disk.qcow2".to_string()),
            size: None,
            format: DiskFormat::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Source {
    #[serde(rename = "web")]
    Web(WebSource),
    #[serde(rename = "file_name")]
    FileName(String),
    #[serde(rename = "custom")]
    // Quickget will be required to manually handle "custom" sources.
    Custom,
    #[serde(rename = "docker")]
    Docker(DockerSource),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DockerSource {
    pub url: String,
    pub privileged: bool,
    pub shared_dirs: Vec<String>,
    pub output_filename: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebSource {
    pub url: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive_format: Option<ArchiveFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
}
impl WebSource {
    pub fn url_only(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            checksum: None,
            archive_format: None,
            file_name: None,
        }
    }
    pub fn new(url: String, checksum: Option<String>, archive_format: Option<ArchiveFormat>, file_name: Option<String>) -> Self {
        Self {
            url,
            checksum,
            archive_format,
            file_name,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum ArchiveFormat {
    #[serde(rename = "tar")]
    Tar,
    #[serde(rename = "tar.bz2")]
    TarBz2,
    #[serde(rename = "tar.gz")]
    TarGz,
    #[serde(rename = "tar.xz")]
    TarXz,
    #[serde(rename = "xz")]
    Xz,
    #[serde(rename = "gz")]
    Gz,
    #[serde(rename = "bz2")]
    Bz2,
    #[serde(rename = "zip")]
    Zip,
    #[serde(rename = "7z")]
    SevenZip,
}
