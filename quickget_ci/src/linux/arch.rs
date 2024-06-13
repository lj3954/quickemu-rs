use crate::{
    store_data::{Config, Distro, Source, WebSource},
    utils::capture_page,
};
use quickget_ci::Arch;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;

const ARCHCRAFT_MIRROR: &str = "https://sourceforge.net/projects/archcraft/files/";

pub struct Archcraft;
impl Distro for Archcraft {
    const NAME: &'static str = "archcraft";
    const PRETTY_NAME: &'static str = "Archcraft";
    const HOMEPAGE: Option<&'static str> = Some("https://archcraft.io/");
    const DESCRIPTION: Option<&'static str> = Some("Yet another minimal Linux distribution, based on Arch Linux.");
    async fn generate_configs() -> Vec<Config> {
        let Some(releases) = capture_page(ARCHCRAFT_MIRROR).await else {
            return Vec::new();
        };
        let releases_regex = Regex::new(r#""name":"v([^"]+)""#).unwrap();
        let url_regex = Arc::new(Regex::new(r#""name":"archcraft-.*?-x86_64.iso".*?"download_url":"([^"]+)".*?"name":"archcraft-.*?-x86_64.iso.sha256sum".*?"download_url":"([^"]+)""#).unwrap());
        let futures = releases_regex.captures_iter(&releases).take(3).map(|r| {
            let release = r[1].to_string();
            let mirror = format!("{ARCHCRAFT_MIRROR}v{release}/");
            let url_regex = url_regex.clone();
            tokio::spawn(async move {
                let page = capture_page(&mirror).await?;
                let urls = url_regex.captures(&page)?;
                let download_url = urls[1].to_string();
                let checksum_url = &urls[2];
                let checksum = capture_page(checksum_url)
                    .await
                    .and_then(|c| c.split_whitespace().next().map(ToString::to_string));
                Some(Config {
                    release: Some(release),
                    edition: None,
                    arch: Arch::x86_64,
                    iso: Some(vec![Source::Web(WebSource::new(download_url, checksum, None, None))]),
                    ..Default::default()
                })
            })
        });
        futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .collect()
    }
}

const ARCHLINUX_API: &str = "https://archlinux.org/releng/releases/json/";
const ARCHLINUX_MIRROR: &str = "https://mirror.rackspace.com/archlinux";

pub struct ArchLinux;
impl Distro for ArchLinux {
    const NAME: &'static str = "archlinux";
    const PRETTY_NAME: &'static str = "Arch Linux";
    const HOMEPAGE: Option<&'static str> = Some("https://archlinux.org/");
    const DESCRIPTION: Option<&'static str> = Some("Lightweight and flexible Linux® distribution that tries to Keep It Simple.");
    async fn generate_configs() -> Vec<Config> {
        let Some(data) = capture_page(ARCHLINUX_API).await else {
            return Vec::new();
        };
        let api_data: ArchAPI = serde_json::from_str(&data).unwrap();
        api_data
            .releases
            .into_iter()
            .take(3)
            .map(|r| {
                let download_url = format!("{ARCHLINUX_MIRROR}{}", r.iso_url);
                let checksum = r.sha256_sum;
                let release = if r.version == api_data.latest_version { "latest".to_string() } else { r.version };
                Config {
                    release: Some(release),
                    edition: None,
                    arch: Arch::x86_64,
                    iso: Some(vec![Source::Web(WebSource::new(download_url, checksum, None, None))]),
                    ..Default::default()
                }
            })
            .collect()
    }
}

#[derive(Deserialize)]
struct ArchAPI {
    releases: Vec<ArchRelease>,
    latest_version: String,
}

#[derive(Deserialize)]
struct ArchRelease {
    version: String,
    sha256_sum: Option<String>,
    iso_url: String,
}

const ARCOLINUX_MIRROR: &str = "https://mirror.accum.se/mirror/arcolinux.info/iso/";

pub struct ArcoLinux;
impl Distro for ArcoLinux {
    const NAME: &'static str = "arcolinux";
    const PRETTY_NAME: &'static str = "ArcoLinux";
    const HOMEPAGE: Option<&'static str> = Some("https://arcolinux.com/");
    const DESCRIPTION: Option<&'static str> = Some("It's all about becoming an expert in Linux.");
    async fn generate_configs() -> Vec<Config> {
        let Some(releases) = capture_page(ARCOLINUX_MIRROR).await else {
            return Vec::new();
        };
        let release_regex = Regex::new(r#">(v[0-9.]+)/</a"#).unwrap();
        let iso_regex = Arc::new(Regex::new(r#">(arco([^-]+)-[v0-9.]+-x86_64.iso)</a>"#).unwrap());
        let checksum_regex = Arc::new(Regex::new(r#">(arco([^-]+)-[v0-9.]+-x86_64.iso.sha256)</a>"#).unwrap());

        let mut releases = release_regex.captures_iter(&releases).collect::<Vec<_>>();
        releases.reverse();
        let futures = releases
            .into_iter()
            .take(3)
            .map(|c| {
                let release = c[1].to_string();
                let mirror = format!("{ARCOLINUX_MIRROR}{release}/");
                let iso_regex = iso_regex.clone();
                let checksum_regex = checksum_regex.clone();
                tokio::spawn(async move {
                    let page = capture_page(&mirror).await?;
                    let checksums = checksum_regex
                        .captures_iter(&page)
                        .map(|c| (c[2].to_string(), c[1].to_string()))
                        .collect::<HashMap<String, String>>();

                    let futures = iso_regex
                        .captures_iter(&page)
                        .filter(|i| !i[2].contains("linux"))
                        .map(|i| {
                            let iso = i[1].to_string();
                            let edition = i[2].to_string();
                            let download_url = format!("{mirror}{iso}");
                            let checksum_url = checksums.get(edition.as_str()).map(|c| format!("{mirror}{c}"));
                            let release = release.clone();
                            tokio::spawn(async move {
                                let checksum = if let Some(checksum_url) = checksum_url {
                                    capture_page(&checksum_url)
                                        .await
                                        .and_then(|c| c.split_whitespace().next().map(ToString::to_string))
                                } else {
                                    None
                                };
                                Config {
                                    release: Some(release),
                                    edition: Some(edition),
                                    iso: Some(vec![Source::Web(WebSource::new(download_url, checksum, None, None))]),
                                    ..Default::default()
                                }
                            })
                        })
                        .collect::<Vec<_>>();
                    Some(futures::future::join_all(futures).await)
                })
            })
            .collect::<Vec<_>>();
        futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .flatten()
            .flatten()
            .collect()
    }
}
