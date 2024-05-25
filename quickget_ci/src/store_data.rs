use quickemu::config::{Arch, GuestOS};
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
    #[serde(default = "default_disk")]
    pub disk_images: Vec<Disk>,
}

#[derive(Serialize, Deserialize)]
pub struct Disk {
    pub source: Source,
    pub size: Option<u64>,
    pub format: DiskFormat,
}
fn default_disk() -> Vec<Disk> {
    vec![Disk {
        source: Source::FileName("disk.qcow2".to_string()),
        size: None,
        format: DiskFormat::Qcow2,
    }]
}

#[derive(Serialize, Deserialize)]
pub enum DiskFormat {
    #[serde(rename = "raw")]
    Raw,
    #[serde(rename = "qcow2")]
    Qcow2,
}

#[derive(Serialize, Deserialize)]
pub enum Source {
    #[serde(rename = "web")]
    Web(WebSource),
    #[serde(rename = "file_name")]
    FileName(String),
    #[serde(rename = "custom")]
    // Quickget will be required to manually handle "custom" sources.
    Custom
}

#[derive(Serialize, Deserialize)]
pub struct WebSource {
    pub url: String,
    pub checksum: Option<String>,
    pub archive_format: Option<ArchiveFormat>,
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
