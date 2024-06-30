use crate::{
    store_data::{Config, Distro, Source, WebSource},
    utils::capture_page,
};
use regex::Regex;

const BIGLINUX_MIRROR: &str = "https://iso.biglinux.com.br/";

pub struct BigLinux;
impl Distro for BigLinux {
    const NAME: &'static str = "biglinux";
    const PRETTY_NAME: &'static str = "BigLinux";
    const HOMEPAGE: Option<&'static str> = Some("https://www.biglinux.com.br/");
    const DESCRIPTION: Option<&'static str> = Some(
        "It's the right choice if you want to have an easy and enriching experience with Linux. It has been perfected over more than 19 years, following our motto: 'In search of the perfect system'",
    );
    async fn generate_configs() -> Option<Vec<Config>> {
        let data = capture_page(BIGLINUX_MIRROR).await?;
        let biglinux_regex = Regex::new(r#"<a href="(biglinux_([0-9]{4}(?:-[0-9]{2}){2})_(.*?).iso)""#).unwrap();

        let mut data = biglinux_regex.captures_iter(&data).collect::<Vec<_>>();
        data.sort_unstable_by_key(|c| c[2].to_string());
        data.reverse();

        let futures = data.into_iter().map(|c| {
            let iso = format!("{BIGLINUX_MIRROR}{}", &c[1]);
            let checksum_url = iso.clone() + ".md5";
            let release = c[2].to_string();
            let edition = c[3].to_string();
            async move {
                let checksum = capture_page(&checksum_url)
                    .await
                    .and_then(|s| s.split_whitespace().next().map(ToString::to_string));
                Config {
                    release: Some(release),
                    edition: Some(edition),
                    iso: Some(vec![Source::Web(WebSource::new(iso, checksum, None, None))]),
                    ..Default::default()
                }
            }
        });

        futures::future::join_all(futures).await.into()
    }
}
