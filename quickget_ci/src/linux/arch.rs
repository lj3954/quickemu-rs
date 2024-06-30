pub mod manjaro;

use crate::{
    store_data::{Config, Distro, Source, WebSource},
    utils::{capture_page, GatherData, GithubAPI},
};
use regex::Regex;
use serde::Deserialize;
use std::{collections::HashMap, sync::Arc};

const ARCHCRAFT_MIRROR: &str = "https://sourceforge.net/projects/archcraft/files/";

pub struct Archcraft;
impl Distro for Archcraft {
    const NAME: &'static str = "archcraft";
    const PRETTY_NAME: &'static str = "Archcraft";
    const HOMEPAGE: Option<&'static str> = Some("https://archcraft.io/");
    const DESCRIPTION: Option<&'static str> = Some("Yet another minimal Linux distribution, based on Arch Linux.");
    async fn generate_configs() -> Option<Vec<Config>> {
        let releases = capture_page(ARCHCRAFT_MIRROR).await?;
        let releases_regex = Regex::new(r#""name":"v([^"]+)""#).unwrap();
        let url_regex = Arc::new(Regex::new(r#""name":"archcraft-.*?-x86_64.iso".*?"download_url":"([^"]+)".*?"name":"archcraft-.*?-x86_64.iso.sha256sum".*?"download_url":"([^"]+)""#).unwrap());
        let futures = releases_regex.captures_iter(&releases).take(3).map(|r| {
            let release = r[1].to_string();
            let mirror = format!("{ARCHCRAFT_MIRROR}v{release}/");
            let url_regex = url_regex.clone();
            async move {
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
                    iso: Some(vec![Source::Web(WebSource::new(download_url, checksum, None, None))]),
                    ..Default::default()
                })
            }
        });
        futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<Config>>()
            .into()
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
    async fn generate_configs() -> Option<Vec<Config>> {
        let data = capture_page(ARCHLINUX_API).await?;
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
                    iso: Some(vec![Source::Web(WebSource::new(download_url, checksum, None, None))]),
                    ..Default::default()
                }
            })
            .collect::<Vec<Config>>()
            .into()
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
    async fn generate_configs() -> Option<Vec<Config>> {
        let releases = capture_page(ARCOLINUX_MIRROR).await?;
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
                async move {
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
                            async move {
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
                            }
                        })
                        .collect::<Vec<_>>();
                    Some(futures::future::join_all(futures).await)
                }
            })
            .collect::<Vec<_>>();
        futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<Config>>()
            .into()
    }
}

const ARTIX_MIRROR: &str = "https://mirrors.ocf.berkeley.edu/artix-iso/";

pub struct ArtixLinux;
impl Distro for ArtixLinux {
    const NAME: &'static str = "artixlinux";
    const PRETTY_NAME: &'static str = "Artix Linux";
    const HOMEPAGE: Option<&'static str> = Some("https://artixlinux.org/");
    const DESCRIPTION: Option<&'static str> = Some("The Art of Linux. Simple. Fast. Systemd-free.");
    async fn generate_configs() -> Option<Vec<Config>> {
        let page = capture_page(ARTIX_MIRROR).await?;
        let iso_regex = Regex::new(r#"href="(artix-(.*?)-([^-]+-[0-9]+)-x86_64.iso)""#).unwrap();

        let checksums = capture_page(&format!("{ARTIX_MIRROR}sha256sums")).await.map(|c| {
            c.lines()
                .filter_map(|l| l.split_once("  ").map(|(hash, file)| (file.to_string(), hash.to_string())))
                .collect::<HashMap<String, String>>()
        });

        iso_regex
            .captures_iter(&page)
            .map(|c| {
                let iso = c[1].to_string();
                let edition = c[2].to_string();
                let release = c[3].to_string();
                let download_url = format!("{ARTIX_MIRROR}{iso}");
                let checksum = checksums.as_ref().and_then(|cs| cs.get(&iso)).map(ToString::to_string);
                Config {
                    release: Some(release),
                    edition: Some(edition),
                    iso: Some(vec![Source::Web(WebSource::new(download_url, checksum, None, None))]),
                    ..Default::default()
                }
            })
            .collect::<Vec<Config>>()
            .into()
    }
}

const ATHENA_API: &str = "https://api.github.com/repos/Athena-OS/athena/releases";

pub struct AthenaOS;
impl Distro for AthenaOS {
    const NAME: &'static str = "athenaos";
    const PRETTY_NAME: &'static str = "Athena OS";
    const HOMEPAGE: Option<&'static str> = Some("https://athenaos.org/");
    const DESCRIPTION: Option<&'static str> = Some("Offer a different experience than the most used pentesting distributions by providing only tools that fit with the user needs and improving the access to hacking resources and learning materials.");
    async fn generate_configs() -> Option<Vec<Config>> {
        let api_data = GithubAPI::gather_data(ATHENA_API).await?;

        let futures = api_data.into_iter().take(2).map(|mut d| async move {
            if d.assets.is_empty() {
                return None;
            }
            let mut release = d.tag_name;
            if d.prerelease {
                release.push_str("-pre");
            }
            let iso_index = d.assets.iter().position(|a| a.name.ends_with(".iso"))?;

            let checksum_name = std::mem::take(&mut d.assets[iso_index].name) + ".sha256";
            let checksum = {
                let checksum_asset = d.assets.iter().find(|a| a.name == checksum_name);
                match checksum_asset {
                    Some(c) => capture_page(&c.browser_download_url)
                        .await
                        .and_then(|c| c.split_whitespace().next().map(ToString::to_string)),
                    None => None,
                }
            };
            let iso_url = d.assets.remove(iso_index).browser_download_url;

            Some(Config {
                release: Some(release),
                iso: Some(vec![Source::Web(WebSource::new(iso_url, checksum, None, None))]),
                ..Default::default()
            })
        });

        futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .collect::<Vec<Config>>()
            .into()
    }
}

pub struct BlendOS;
impl Distro for BlendOS {
    const NAME: &'static str = "blendos";
    const PRETTY_NAME: &'static str = "BlendOS";
    const HOMEPAGE: Option<&'static str> = Some("https://blendos.co/");
    const DESCRIPTION: Option<&'static str> = Some(
        "A seamless blend of all Linux distributions. Allows you to have an immutable, atomic and declarative Arch Linux system, with application support from several Linux distributions & Android.",
    );
    async fn generate_configs() -> Option<Vec<Config>> {
        Some(vec![Config {
            iso: Some(vec![Source::Web(WebSource::url_only(
                "https://kc1.mirrors.199693.xyz/blend/isos/testing/blendOS.iso",
            ))]),
            ..Default::default()
        }])
    }
}
