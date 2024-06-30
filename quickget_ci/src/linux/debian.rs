use crate::{
    store_data::{Config, Distro, Source, WebSource},
    utils::capture_page,
};
use regex::Regex;
use std::collections::HashMap;
use std::sync::Arc;

const ANTIX_MIRROR: &str = "https://sourceforge.net/projects/antix-linux/files/Final/";

pub struct Antix;
impl Distro for Antix {
    const NAME: &'static str = "antix";
    const PRETTY_NAME: &'static str = "antiX";
    const HOMEPAGE: Option<&'static str> = Some("https://antixlinux.com/");
    const DESCRIPTION: Option<&'static str> = Some("Fast, lightweight and easy to install systemd-free linux live CD distribution based on Debian Stable for Intel-AMD x86 compatible systems.");
    async fn generate_configs() -> Option<Vec<Config>> {
        let releases = capture_page(ANTIX_MIRROR).await?;

        let releases_regex = Regex::new(r#""name":"antiX-([0-9.]+)""#).unwrap();
        let iso_regex = Arc::new(Regex::new(r#""name":"(antiX-[0-9.]+(?:-runit)?(?:-[^_]+)?_x64-([^.]+).iso)".*?"download_url":"(.*?)""#).unwrap());

        let futures = releases_regex.captures_iter(&releases).take(3).map(|r| {
            let release = r[1].to_string();
            let mirror = format!("{ANTIX_MIRROR}antiX-{release}/");
            let checksum_mirror = format!("{mirror}README.txt/download");
            let runit_mirror = format!("{mirror}runit-antiX-{release}/");
            let runit_checksum_mirror = format!("{runit_mirror}README2.txt/download");
            let iso_regex = iso_regex.clone();

            async move {
                let main_checksums = capture_page(&checksum_mirror).await;
                let runit_checksums = capture_page(&runit_checksum_mirror).await;
                let mut checksums = main_checksums
                    .iter()
                    .chain(runit_checksums.iter())
                    .flat_map(|cs| {
                        cs.lines()
                            .skip_while(|l| !l.starts_with("sha256"))
                            .filter_map(|l| l.split_once("  ").map(|(a, b)| (b.to_string(), a.to_string())))
                    })
                    .collect::<HashMap<String, String>>();

                let page = capture_page(&mirror).await?;
                let iso_regex = iso_regex.clone();
                let main_releases = iso_regex.captures_iter(&page).zip(std::iter::repeat("-sysv"));
                let runit_page = capture_page(&runit_mirror).await?;
                let runit_releases = iso_regex.captures_iter(&runit_page).zip(std::iter::repeat("-runit"));

                Some(
                    main_releases
                        .chain(runit_releases)
                        .map(|(c, ending)| {
                            let checksum = checksums.remove(&c[1]);
                            let edition = c[2].to_string() + ending;
                            let url = c[3].to_string();
                            Config {
                                release: Some(release.clone()),
                                edition: Some(edition),
                                iso: Some(vec![Source::Web(WebSource::new(url, checksum, None, None))]),
                                ..Default::default()
                            }
                        })
                        .collect::<Vec<_>>(),
                )
            }
        });

        futures::future::join_all(futures)
            .await
            .into_iter()
            .flatten()
            .flatten()
            .collect::<Vec<Config>>()
            .into()
    }
}
