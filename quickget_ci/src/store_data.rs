use crate::utils::all_valid;
pub use quickemu::config::Arch;
pub use quickget::data_structures::{ArchiveFormat, Config, Disk, Source, WebSource, OS};

pub trait Distro {
    const NAME: &'static str;
    const PRETTY_NAME: &'static str;
    const HOMEPAGE: Option<&'static str>;
    const DESCRIPTION: Option<&'static str>;
    async fn generate_configs() -> Option<Vec<Config>>;
}

pub trait ToOS {
    #![allow(dead_code)]
    async fn to_os(&self) -> Option<OS>;
}

impl<T: Distro + Send> ToOS for T {
    async fn to_os(&self) -> Option<OS> {
        // Any entry containing a URL which isn't reachable needs to be removed
        let Some(releases) = Self::generate_configs().await else {
            log::error!("Failed to generate configs for {}", Self::PRETTY_NAME);
            return None;
        };
        if releases.is_empty() {
            log::error!("No releases found for {}", Self::PRETTY_NAME);
            return None;
        }
        let futures = releases.iter().map(|r| {
            let urls = [
                filter_web_sources(r.iso.as_deref()),
                filter_web_sources(r.img.as_deref()),
                filter_web_sources(r.fixed_iso.as_deref()),
                filter_web_sources(r.floppy.as_deref()),
                extract_disk_urls(r.disk_images.as_deref()),
            ]
            .concat();
            async move { all_valid(urls).await }
        });
        let results = futures::future::join_all(futures).await;
        let releases = releases
            .into_iter()
            .zip(results)
            .filter_map(|(config, valid)| {
                if valid {
                    Some(config)
                } else {
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

        Some(OS {
            name: Self::NAME.into(),
            pretty_name: Self::PRETTY_NAME.into(),
            homepage: Self::HOMEPAGE.map(Into::into),
            description: Self::DESCRIPTION.map(Into::into),
            releases,
        })
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
