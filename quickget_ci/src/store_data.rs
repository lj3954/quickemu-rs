use quickemu::config::{Arch, DiskFormat, GuestOS};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OS {
    pub name: String,
    pub pretty_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub releases: Vec<Config>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edition: Option<String>,
    #[serde(skip_serializing_if = "is_default")]
    pub guest_os: GuestOS,
    #[serde(skip_serializing_if = "is_default")]
    pub arch: Arch,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub iso: Option<Vec<Source>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub img: Option<Vec<Source>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_iso: Option<Vec<Source>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floppy: Option<Vec<Source>>,
    #[serde(skip_serializing_if = "is_default_disk")]
    pub disk_images: Option<Vec<Disk>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            release: None,
            edition: None,
            guest_os: GuestOS::Linux,
            arch: Arch::x86_64,
            iso: None,
            img: None,
            fixed_iso: None,
            floppy: None,
            disk_images: Some(vec![Default::default()]),
        }
    }
}
fn is_default_disk(disk: &Option<Vec<Disk>>) -> bool {
    &Some(vec![Default::default()]) == disk
}
fn is_default<T: Default + PartialEq>(input: &T) -> bool {
    input == &T::default()
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct Disk {
    pub source: Source,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u64>,
    #[serde(skip_serializing_if = "is_default")]
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

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Source {
    #[serde(rename = "web")]
    Web(WebSource),
    #[serde(rename = "file_name")]
    FileName(String),
    #[serde(rename = "custom")]
    // Quickget will be required to manually handle "custom" sources.
    Custom,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub struct WebSource {
    url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    checksum: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    archive_format: Option<ArchiveFormat>,
}
impl WebSource {
    pub fn url_only(url: String) -> Self {
        Self {
            url,
            checksum: None,
            archive_format: None,
        }
    }
    pub fn new(url: String, checksum: Option<String>, archive_format: Option<ArchiveFormat>) -> Self {
        Self { url, checksum, archive_format }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
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
}

pub trait Distro {
    const NAME: &'static str;
    const PRETTY_NAME: &'static str;
    const HOMEPAGE: Option<&'static str>;
    const DESCRIPTION: Option<&'static str>;
    fn generate_configs(&self) -> Vec<Config>;
}

impl<T: Distro> From<T> for OS {
    fn from(distro: T) -> Self {
        OS {
            name: T::NAME.into(),
            pretty_name: T::PRETTY_NAME.into(),
            homepage: T::HOMEPAGE.map(|s| s.to_string()),
            description: T::DESCRIPTION.map(|s| s.to_string()),
            releases: distro.generate_configs(),
        }
    }
}
