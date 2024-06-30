use crate::{
    store_data::{Config, Distro, Source, WebSource},
    utils::capture_page,
};
use once_cell::sync::Lazy;
use regex::Regex;
use serde::Deserialize;
use std::sync::Arc;
use tokio::runtime::Runtime;

const LAUNCHPAD_RELEASES_URL: &str = "https://api.launchpad.net/devel/ubuntu/series";

pub struct Ubuntu;
impl Distro for Ubuntu {
    const NAME: &'static str = "ubuntu";
    const PRETTY_NAME: &'static str = "Ubuntu";
    const HOMEPAGE: Option<&'static str> = Some("https://www.ubuntu.com/");
    const DESCRIPTION: Option<&'static str> = Some("Complete desktop Linux operating system, freely available with both community and professional support.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::Ubuntu).await
    }
}

pub struct UbuntuServer;
impl Distro for UbuntuServer {
    const NAME: &'static str = "ubuntu-server";
    const PRETTY_NAME: &'static str = "Ubuntu Server";
    const HOMEPAGE: Option<&'static str> = Some("https://www.ubuntu.com/server");
    const DESCRIPTION: Option<&'static str> = Some("Brings economic and technical scalability to your datacentre, public or private. Whether you want to deploy an OpenStack cloud, a Kubernetes cluster or a 50,000-node render farm, Ubuntu Server delivers the best value scale-out performance available.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::UbuntuServer).await
    }
}

pub struct UbuntuUnity;
impl Distro for UbuntuUnity {
    const NAME: &'static str = "ubuntu-unity";
    const PRETTY_NAME: &'static str = "Ubuntu Unity";
    const HOMEPAGE: Option<&'static str> = Some("https://ubuntuunity.org/");
    const DESCRIPTION: Option<&'static str> = Some("Flavor of Ubuntu featuring the Unity7 desktop environment (the default desktop environment used by Ubuntu from 2010-2017).");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::UbuntuUnity).await
    }
}

pub struct UbuntuStudio;
impl Distro for UbuntuStudio {
    const NAME: &'static str = "ubuntu-studio";
    const PRETTY_NAME: &'static str = "Ubuntu Studio";
    const HOMEPAGE: Option<&'static str> = Some("https://ubuntustudio.org/");
    const DESCRIPTION: Option<&'static str> = Some("Comes preinstalled with a selection of the most common free multimedia applications available, and is configured for best performance for various purposes: Audio, Graphics, Video, Photography and Publishing.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::UbuntuStudio).await
    }
}

pub struct UbuntuMATE;
impl Distro for UbuntuMATE {
    const NAME: &'static str = "ubuntu-mate";
    const PRETTY_NAME: &'static str = "Ubuntu MATE";
    const HOMEPAGE: Option<&'static str> = Some("https://ubuntu-mate.org/");
    const DESCRIPTION: Option<&'static str> =
        Some("Stable, easy-to-use operating system with a configurable desktop environment. It is ideal for those who want the most out of their computers and prefer a traditional desktop metaphor.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::UbuntuMATE).await
    }
}

pub struct UbuntuBudgie;
impl Distro for UbuntuBudgie {
    const NAME: &'static str = "ubuntu-budgie";
    const PRETTY_NAME: &'static str = "Ubuntu Budgie";
    const HOMEPAGE: Option<&'static str> = Some("https://ubuntubudgie.org/");
    const DESCRIPTION: Option<&'static str> = Some("Community developed distribution, integrating the Budgie Desktop Environment with Ubuntu at its core.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::UbuntuBudgie).await
    }
}

pub struct Lubuntu;
impl Distro for Lubuntu {
    const NAME: &'static str = "lubuntu";
    const PRETTY_NAME: &'static str = "Lubuntu";
    const HOMEPAGE: Option<&'static str> = Some("https://lubuntu.me/");
    const DESCRIPTION: Option<&'static str> =
        Some("Complete Operating System that ships the essential apps and services for daily use: office applications, PDF reader, image editor, music and video players, etc.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::Lubuntu).await
    }
}

pub struct Kubuntu;
impl Distro for Kubuntu {
    const NAME: &'static str = "kubuntu";
    const PRETTY_NAME: &'static str = "Kubuntu";
    const HOMEPAGE: Option<&'static str> = Some("https://kubuntu.org/");
    const DESCRIPTION: Option<&'static str> = Some("Free, complete, and open-source alternative to Microsoft Windows and Mac OS X which contains everything you need to work, play, or share.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::Kubuntu).await
    }
}

pub struct Xubuntu;
impl Distro for Xubuntu {
    const NAME: &'static str = "xubuntu";
    const PRETTY_NAME: &'static str = "Xubuntu";
    const HOMEPAGE: Option<&'static str> = Some("https://xubuntu.org/");
    const DESCRIPTION: Option<&'static str> = Some("Elegant and easy to use operating system. Xubuntu comes with Xfce, which is a stable, light and configurable desktop environment.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::Xubuntu).await
    }
}

pub struct Edubuntu;
impl Distro for Edubuntu {
    const NAME: &'static str = "edubuntu";
    const PRETTY_NAME: &'static str = "Edubuntu";
    const HOMEPAGE: Option<&'static str> = Some("https://www.edubuntu.org/");
    const DESCRIPTION: Option<&'static str> = Some("Stable, secure and privacy concious option for schools.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::Edubuntu).await
    }
}

pub struct UbuntuCinnamon;
impl Distro for UbuntuCinnamon {
    const NAME: &'static str = "ubuntu-cinnamon";
    const PRETTY_NAME: &'static str = "Ubuntu Cinnamon";
    const HOMEPAGE: Option<&'static str> = Some("https://ubuntucinnamon.org/");
    const DESCRIPTION: Option<&'static str> =
        Some("Community-driven, featuring Linux Mint’s Cinnamon Desktop with Ubuntu at the core, packed fast and full of features, here is the most traditionally modern desktop you will ever love.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::UbuntuCinnamon).await
    }
}

pub struct UbuntuKylin;
impl Distro for UbuntuKylin {
    const NAME: &'static str = "ubuntu-kylin";
    const PRETTY_NAME: &'static str = "Ubuntu Kylin";
    const HOMEPAGE: Option<&'static str> = Some("https://www.ubuntukylin.com/");
    const DESCRIPTION: Option<&'static str> =
        Some("Universal desktop operating system for personal computers, laptops, and embedded devices. It is dedicated to bringing a smarter user experience to users all over the world.");
    async fn generate_configs() -> Option<Vec<Config>> {
        get_ubuntu_releases(UbuntuVariant::UbuntuKylin).await
    }
}

async fn get_ubuntu_releases(variant: UbuntuVariant) -> Option<Vec<Config>> {
    let futures = UBUNTU_RELEASES.iter().map(|release| {
        let url = match (release.as_str(), &variant) {
            ("daily-live", _) => format!("https://cdimage.ubuntu.com/{}/{release}/current/", variant.as_ref()),
            (_, UbuntuVariant::Ubuntu | UbuntuVariant::UbuntuServer) => format!("https://releases.ubuntu.com/{release}/"),
            _ => format!("https://cdimage.ubuntu.com/{}/releases/{release}/release/", variant.as_ref()),
        };

        let sku = match variant {
            UbuntuVariant::UbuntuServer => "live-server",
            UbuntuVariant::UbuntuStudio => "dvd",
            _ => "desktop",
        };
        async move {
            let text = match capture_page(&format!("{}SHA256SUMS", url)).await {
                Some(text) => text,
                None => capture_page(&format!("{}MD5SUMS", url)).await?,
            };

            let line = text.lines().find(|l| l.contains("amd64") && l.contains(sku))?;
            let hash = line.split_whitespace().next();
            let iso = format!("{url}{}", line.split('*').nth(1)?);

            Some(Config {
                iso: Some(vec![Source::Web(WebSource::new(iso, hash.map(Into::into), None, None))]),
                release: Some(release.to_string()),
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

enum UbuntuVariant {
    Ubuntu,
    UbuntuServer,
    UbuntuUnity,
    Lubuntu,
    Kubuntu,
    UbuntuMATE,
    UbuntuBudgie,
    UbuntuStudio,
    UbuntuKylin,
    Edubuntu,
    Xubuntu,
    UbuntuCinnamon,
}

impl AsRef<str> for UbuntuVariant {
    fn as_ref(&self) -> &str {
        match self {
            UbuntuVariant::Ubuntu => "ubuntu",
            UbuntuVariant::UbuntuServer => "ubuntu-server",
            UbuntuVariant::UbuntuUnity => "ubuntu-unity",
            UbuntuVariant::Lubuntu => "lubuntu",
            UbuntuVariant::Kubuntu => "kubuntu",
            UbuntuVariant::UbuntuMATE => "ubuntu-mate",
            UbuntuVariant::UbuntuBudgie => "ubuntu-budgie",
            UbuntuVariant::UbuntuStudio => "ubuntustudio",
            UbuntuVariant::UbuntuKylin => "ubuntukylin",
            UbuntuVariant::Edubuntu => "edubuntu",
            UbuntuVariant::Xubuntu => "xubuntu",
            UbuntuVariant::UbuntuCinnamon => "ubuntucinnamon",
        }
    }
}

#[derive(Deserialize)]
struct LaunchpadContents {
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
struct Entry {
    version: String,
    status: String,
}

const ELEMENTARY_URL: &str = "https://elementary.io/";
const ELEMENTARY_CHECKSUM_URL: &str = "https://elementary.io/docs/installation";

pub struct Elementary;
impl Distro for Elementary {
    const NAME: &'static str = "elementary";
    const PRETTY_NAME: &'static str = "elementary OS";
    const HOMEPAGE: Option<&'static str> = Some("https://elementary.io/");
    const DESCRIPTION: Option<&'static str> = Some("Thoughtful, capable, and ethical replacement for Windows and macOS.");
    async fn generate_configs() -> Option<Vec<Config>> {
        let download_regex = Regex::new(r#"download-link http" href="(.*?)">Download"#).unwrap();
        let checksum_regex = Regex::new(r#""language-bash">([0-9a-f]{64})</code>"#).unwrap();

        let page = capture_page(ELEMENTARY_URL).await?;
        let dl_link = download_regex.captures(&page).map(|c| "https:".to_string() + &c[1])?;

        let checksum = capture_page(ELEMENTARY_CHECKSUM_URL)
            .await
            .and_then(|html| checksum_regex.captures(&html).map(|c| c[1].to_string()));

        vec![Config {
            iso: Some(vec![Source::Web(WebSource::new(dl_link, checksum, None, None))]),
            ..Default::default()
        }]
        .into()
    }
}

const BODHI_MIRROR: &str = "https://sourceforge.net/projects/bodhilinux/files/";

pub struct Bodhi;
impl Distro for Bodhi {
    const NAME: &'static str = "bodhi";
    const PRETTY_NAME: &'static str = "Bodhi";
    const HOMEPAGE: Option<&'static str> = Some("https://www.bodhilinux.com/");
    const DESCRIPTION: Option<&'static str> = Some("Lightweight distribution featuring the fast & fully customizable Moksha Desktop.");
    async fn generate_configs() -> Option<Vec<Config>> {
        let page = capture_page(BODHI_MIRROR).await?;
        let release_regex = Regex::new(r#""name":"([0-9]+.[0-9]+.[0-9]+)""#).unwrap();
        let iso_regex = Arc::new(Regex::new(r#""name":"(bodhi-[0-9]+.[0-9]+.[0-9]+-64(-[^-.]+)?.iso)""#).unwrap());

        let futures = release_regex.captures_iter(&page).take(3).map(|c| {
            let release = c[1].to_string();
            let mirror = format!("{BODHI_MIRROR}{release}/");
            let iso_regex = iso_regex.clone();
            async move {
                let release_page = capture_page(&mirror).await?;
                let futures = iso_regex.captures_iter(&release_page).map(|c| {
                    let release = release.clone();
                    let edition = c.get(2).map(|m| m.as_str()[1..].to_string()).unwrap_or("standard".to_string());
                    let iso = format!("{mirror}{}/download", &c[1]);
                    let checksum_url = format!("{mirror}{}.sha256/download", &c[1]);
                    async move {
                        let checksum = capture_page(&checksum_url)
                            .await
                            .and_then(|c| c.split_whitespace().next().map(Into::into));
                        Config {
                            iso: Some(vec![Source::Web(WebSource::new(iso, checksum, None, None))]),
                            release: Some(release),
                            edition: Some(edition),
                            ..Default::default()
                        }
                    }
                });
                Some(futures::future::join_all(futures).await)
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
