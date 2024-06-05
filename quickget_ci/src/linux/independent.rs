use crate::store_data::{Config, Source, WebSource};
use crate::utils::capture_page;
use quickemu::config::Arch;
use regex::Regex;
use serde::Deserialize;

use crate::store_data::Distro;

const NIX_URL: &str = "https://nix-channels.s3.amazonaws.com/?delimiter=/";
const NIX_DOWNLOAD_URL: &str = "https://channels.nixos.org";

pub struct NixOS;
impl Distro for NixOS {
    const NAME: &'static str = "nixos";
    const PRETTY_NAME: &'static str = "NixOS";
    const HOMEPAGE: Option<&'static str> = Some("https://nixos.org/");
    const DESCRIPTION: Option<&'static str> = Some("Linux distribution based on Nix package manager, tool that takes a unique approach to package management and system configuration.");
    async fn generate_configs() -> Vec<Config> {
        let standard_release = Regex::new(r#"nixos-(([0-9]+.[0-9]+|(unstable))(?:-small)?)"#).unwrap();
        let iso_regex = Regex::new(r#"latest-nixos-([^-]+)-([^-]+)-linux.iso"#).unwrap();
        let Some(releases) = capture_page(NIX_URL)
            .await
            .and_then(|page| quick_xml::de::from_str::<NixReleases>(&page).ok())
        else {
            return Vec::new();
        };

        let releases: Vec<String> = releases
            .contents
            .into_iter()
            .map(|r| r.key)
            .filter(|r| standard_release.is_match(r))
            .rev()
            .take(6)
            .map(|r| standard_release.captures(&r).unwrap().get(1).unwrap().as_str().to_string())
            .collect();
        let mut futures = Vec::new();
        for release in releases {
            if let Some(page) = capture_page(&format!("{NIX_URL}&prefix=nixos-{release}/"))
                .await
                .and_then(|p| quick_xml::de::from_str::<NixReleases>(&p).ok())
            {
                let page = page
                    .contents
                    .into_iter()
                    .map(|r| r.key)
                    .filter(|r| iso_regex.is_match(r) && r.ends_with(".iso"))
                    .collect::<Vec<String>>();

                futures.append(
                    &mut page
                        .into_iter()
                        .map(|page| {
                            let capture = iso_regex.captures(&page).unwrap();
                            let release = release.clone();
                            let name = capture.get(0).map(|n| n.as_str().to_string());
                            let edition = capture.get(1).map(|e| e.as_str().to_string());
                            let arch: Option<Arch> = capture.get(2).map(|a| a.as_str().to_string()).try_into().ok();
                            tokio::spawn(async move {
                                let iso = format!("{NIX_DOWNLOAD_URL}/nixos-{release}/{}", name?);
                                let hash = capture_page(&format!("{iso}.sha256"))
                                    .await
                                    .map(|h| h.split_whitespace().next().unwrap().to_string());
                                Some(Config {
                                    release: Some(release),
                                    edition: Some(edition?),
                                    arch: arch?,
                                    iso: Some(vec![Source::Web(WebSource::new(iso, hash, None, None))]),
                                    ..Default::default()
                                })
                            })
                        })
                        .collect(),
                );
            };
        }
        futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .collect()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NixReleases {
    contents: Vec<NixRelease>,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct NixRelease {
    key: String,
}
