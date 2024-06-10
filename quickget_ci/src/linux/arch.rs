use crate::{
    store_data::{Config, Distro, Source, WebSource},
    utils::capture_page,
};
use quickget_ci::Arch;
use regex::Regex;
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
        let futures = releases_regex.captures_iter(&releases).map(|r| {
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
