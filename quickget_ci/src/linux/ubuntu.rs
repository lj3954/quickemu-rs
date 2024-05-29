use crate::store_data::{Config, Distro};
use crate::utils::capture_page;
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::runtime::Runtime;

const LAUNCHPAD_RELEASES_URL: &str = "https://api.launchpad.net/devel/ubuntu/series";

pub struct Ubuntu {}
impl Distro for Ubuntu {
    const NAME: &'static str = "ubuntu";
    const PRETTY_NAME: &'static str = "Ubuntu";
    const HOMEPAGE: Option<&'static str> = Some("https://www.ubuntu.com/");
    const DESCRIPTION: Option<&'static str> = Some("Complete desktop Linux operating system, freely available with both community and professional support.");
    async fn generate_configs() -> Vec<Config> {
        UBUNTU_RELEASES.iter().for_each(|r| println!("{}", r));
        todo!()
    }
}

static UBUNTU_RELEASES: Lazy<Vec<String>> = Lazy::new(|| {
    let Ok(rt) = Runtime::new() else { return Vec::new() };
    let Ok(text) = std::thread::spawn(move || rt.block_on(async { capture_page(LAUNCHPAD_RELEASES_URL).await })).join() else {
        return Vec::new();
    };

    let entries: Option<LaunchpadContents> = text.and_then(|t| serde_json::from_str(&t).ok());
    let mut releases: Vec<String> = entries
        .map(|page| {
            page.entries
                .into_iter()
                .filter(|e| e.status == "Supported" || e.status == "Current Stable Release")
                .map(|e| e.version)
                .collect()
        })
        .unwrap_or_default();
    releases.push("daily-live".to_string());
    releases
});

#[derive(Deserialize)]
struct LaunchpadContents {
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    version: String,
    status: String,
}
