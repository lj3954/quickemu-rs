#![allow(dead_code)]
use quickemu::config::{Arch, BootType, DiskFormat, GuestOS};
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
    #[serde(default, skip_serializing_if = "is_default")]
    pub guest_os: GuestOS,
    #[serde(default, skip_serializing_if = "is_default")]
    pub arch: Arch,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub iso: Option<Vec<Source>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub img: Option<Vec<Source>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_iso: Option<Vec<Source>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floppy: Option<Vec<Source>>,
    #[serde(default = "default_disk", skip_serializing_if = "is_default_disk")]
    pub disk_images: Option<Vec<Disk>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub boot_type: Option<BootType>,
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
            arch: Arch::x86_64,
            iso: None,
            img: None,
            fixed_iso: None,
            floppy: None,
            disk_images: default_disk(),
            boot_type: None,
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Disk {
    pub source: Source,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub format: DiskFormat,
}
impl Default for Disk {
    fn default() -> Self {
        Self {
            source: Source::FileName("disk.qcow2".to_string()),
            size: None,
            format: DiskFormat::Qcow2,
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
