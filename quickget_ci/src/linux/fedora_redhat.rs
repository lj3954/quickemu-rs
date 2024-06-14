use crate::{
    store_data::{Config, Distro, Source, WebSource},
    utils::capture_page,
};
use quickget_ci::Arch;
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

const ALMA_MIRROR: &str = "https://repo.almalinux.org/almalinux/";

pub struct Alma;
impl Distro for Alma {
    const NAME: &'static str = "alma";
    const PRETTY_NAME: &'static str = "AlmaLinux";
    const HOMEPAGE: Option<&'static str> = Some("https://almalinux.org/");
    const DESCRIPTION: Option<&'static str> = Some("Community owned and governed, forever-free enterprise Linux distribution, focused on long-term stability, providing a robust production-grade platform. AlmaLinux OS is binary compatible with RHEL®.");
    async fn generate_configs() -> Option<Vec<Config>> {
        let releases = capture_page(ALMA_MIRROR).await?;

        let releases_regex = Regex::new(r#"<a href="([0-9]+)/""#).unwrap();
        let iso_regex = Arc::new(Regex::new(r#"<a href="(AlmaLinux-[0-9]+-latest-(?:x86_64|aarch64)-([^-]+).iso)">"#).unwrap());
        let checksum_regex = Arc::new(Regex::new(r#"SHA256 \(([^)]+)\) = ([0-9a-f]+)"#).unwrap());

        let futures = releases_regex.captures_iter(&releases).flat_map(|r| {
            let release = r[1].to_string();
            [Arch::x86_64, Arch::aarch64]
                .iter()
                .map(|arch| {
                    let release = release.clone();
                    let iso_regex = iso_regex.clone();
                    let checksum_regex = checksum_regex.clone();
                    let mirror = format!("{ALMA_MIRROR}{release}/isos/{arch}/");

                    tokio::spawn(async move {
                        let page = capture_page(&mirror).await?;
                        let checksum_page = capture_page(&format!("{mirror}CHECKSUM")).await;
                        let checksums = checksum_page.map(|cs| {
                            checksum_regex
                                .captures_iter(&cs)
                                .map(|c| (c[1].to_string(), c[2].to_string()))
                                .collect::<HashMap<String, String>>()
                        });

                        Some(
                            iso_regex
                                .captures_iter(&page)
                                .filter(|c| !c.get(0).unwrap().as_str().ends_with(".manifest"))
                                .map(|c| {
                                    let iso = c[1].to_string();
                                    let edition = c[2].to_string();
                                    let url = format!("{mirror}{iso}");
                                    let checksum = checksums.as_ref().and_then(|cs| cs.get(&iso)).cloned();
                                    Config {
                                        release: Some(release.to_string()),
                                        edition: Some(edition),
                                        arch: arch.clone(),
                                        iso: Some(vec![Source::Web(WebSource::new(url, checksum, None, None))]),
                                        ..Default::default()
                                    }
                                })
                                .collect::<Vec<Config>>(),
                        )
                    })
                })
                .collect::<Vec<_>>()
        });

        futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .flatten()
            .collect::<Vec<Config>>()
            .into()
    }
}
