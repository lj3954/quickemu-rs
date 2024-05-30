use crate::utils::all_valid;
use quickemu::config::{Arch, DiskFormat, GuestOS};
use serde::{Deserialize, Serialize};
use tokio::spawn;

#[derive(Serialize, Deserialize)]
pub struct OS {
    pub name: &'static str,
    pub pretty_name: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'static str>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    file_name: Option<String>,
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
    async fn generate_configs() -> Vec<Config>;
}

pub trait ToOS {
    #![allow(dead_code)]
    async fn to_os(&self) -> OS;
}

impl<T: Distro + Send> ToOS for T {
    async fn to_os(&self) -> OS {
        // Any entry containing a URL which isn't reachable needs to be removed
        let releases = Self::generate_configs().await;
        let futures = releases.iter().map(|r| {
            let urls = [
                filter_web_sources(r.iso.as_deref()),
                filter_web_sources(r.img.as_deref()),
                filter_web_sources(r.fixed_iso.as_deref()),
                filter_web_sources(r.floppy.as_deref()),
                extract_disk_urls(r.disk_images.as_deref()),
            ]
            .concat();
            spawn(async move { all_valid(urls).await })
        });
        let results = futures::future::join_all(futures).await;
        let releases = releases
            .into_iter()
            .zip(results)
            .filter_map(|(config, valid)| match valid {
                Ok(true) => Some(config),
                _ => {
                    log::warn!(
                        "Removing {} {} {} {} due to unresolvable URL",
                        Self::PRETTY_NAME,
                        config.release.unwrap_or_default(),
                        config.edition.unwrap_or_default(),
                        config.arch
                    );
                    None
                }
            })
            .collect::<Vec<Config>>();

        OS {
            name: Self::NAME,
            pretty_name: Self::PRETTY_NAME,
            homepage: Self::HOMEPAGE,
            description: Self::DESCRIPTION,
            releases,
        }
    }
}

pub fn filter_web_sources(sources: Option<&[Source]>) -> Vec<String> {
    sources
        .unwrap_or(&[])
        .iter()
        .filter_map(|s| match s {
            Source::Web(w) => Some(w.url.clone()),
            _ => None,
        })
        .collect()
}

pub fn extract_disk_urls(sources: Option<&[Disk]>) -> Vec<String> {
    sources
        .unwrap_or(&[])
        .iter()
        .map(|d| &d.source)
        .filter_map(|s| match s {
            Source::Web(w) => Some(w.url.clone()),
            _ => None,
        })
        .collect()
}
