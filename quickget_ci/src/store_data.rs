use quickemu::config::{Arch, DiskFormat, GuestOS};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct OS {
    pub name: String,
    pub pretty_name: String,
    pub homepage: Option<String>,
    pub description: Option<String>,
    pub releases: Vec<Config>,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub release: Option<String>,
    pub edition: Option<String>,
    pub guest_os: GuestOS,
    pub arch: Arch,
    pub iso: Option<Vec<Source>>,
    pub img: Option<Vec<Source>>,
    pub fixed_iso: Option<Vec<Source>>,
    pub floppy: Option<Vec<Source>>,
    pub disk_images: Vec<Disk>,
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
            disk_images: vec![Default::default()],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Disk {
    pub source: Source,
    pub size: Option<u64>,
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

#[derive(Serialize, Deserialize)]
pub enum Source {
    #[serde(rename = "web")]
    Web(WebSource),
    #[serde(rename = "file_name")]
    FileName(String),
    #[serde(rename = "custom")]
    // Quickget will be required to manually handle "custom" sources.
    Custom,
}

#[derive(Serialize, Deserialize)]
pub struct WebSource {
    url: String,
    checksum: Option<String>,
    archive_format: Option<ArchiveFormat>,
}
impl WebSource {
    fn url_only(url: String) -> Self {
        Self {
            url,
            checksum: None,
            archive_format: None,
        }
    }
    fn new(url: String, checksum: Option<String>, archive_format: Option<ArchiveFormat>) -> Self {
        Self { url, checksum, archive_format }
    }
}

#[derive(Serialize, Deserialize)]
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