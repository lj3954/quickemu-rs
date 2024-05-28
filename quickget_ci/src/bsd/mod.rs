use crate::store_data::{ArchiveFormat, Config, Distro, Source, WebSource};
use crate::utils::capture_page;
use quickemu::config::{Arch, GuestOS};
use regex::Regex;

const FREEBSD_X86_64_RELEASES: &str = "https://download.freebsd.org/ftp/releases/amd64/amd64/";
const FREEBSD_AARCH64_RELEASES: &str = "https://download.freebsd.org/ftp/releases/arm64/aarch64/";
const FREEBSD_EDITIONS: [&str; 2] = ["disc1", "dvd1"];

pub struct FreeBSD {}
impl Distro for FreeBSD {
    const NAME: &'static str = "freebsd";
    const PRETTY_NAME: &'static str = "FreeBSD";
    const HOMEPAGE: Option<&'static str> = Some("https://www.freebsd.org/");
    const DESCRIPTION: Option<&'static str> = Some("Operating system used to power modern servers, desktops, and embedded platforms.");
    fn generate_configs(&self) -> Vec<Config> {
        // TODO: Add riscv64
        let mut releases: Vec<Config> = Vec::new();
        let freebsd_regex = Regex::new(r#"href="([0-9\.]+)-RELEASE"#).unwrap();
        let find_checksum = |checksums: Option<&str>, iso: &str| {
            checksums.and_then(|c| {
                c.lines()
                    .find(|l| l.contains(iso))
                    .and_then(|l| l.split_once(" = ").map(|(_, c)| c.to_string()))
            })
        };
        if let Some(page) = capture_page(FREEBSD_X86_64_RELEASES) {
            for capture in freebsd_regex.captures_iter(&page) {
                let release = &capture[1];
                let checksum_url = format!("{FREEBSD_X86_64_RELEASES}ISO-IMAGES/{release}/CHECKSUM.SHA512-FreeBSD-{release}-RELEASE-amd64");
                let checksums = capture_page(&checksum_url);
                for edition in FREEBSD_EDITIONS {
                    let iso = format!("FreeBSD-{release}-RELEASE-amd64-{edition}.iso.xz");
                    let checksum = find_checksum(checksums.as_deref(), &iso);
                    let url = format!("{FREEBSD_X86_64_RELEASES}ISO-IMAGES/{release}/{iso}");
                    releases.push(Config {
                        guest_os: GuestOS::FreeBSD,
                        iso: Some(vec![Source::Web(WebSource::new(url, checksum, Some(ArchiveFormat::Xz), None))]),
                        ..Default::default()
                    });
                }
            }
        } else {
            log::warn!("Failed to fetch FreeBSD x86_64 releases");
        }
        if let Some(page) = capture_page(FREEBSD_AARCH64_RELEASES) {
            for capture in freebsd_regex.captures_iter(&page) {
                let release = &capture[1];
                let checksum_url = format!("{FREEBSD_AARCH64_RELEASES}ISO-IMAGES/{release}/CHECKSUM.SHA512-FreeBSD-{release}-RELEASE-arm64-aarch64");
                let checksums = capture_page(&checksum_url);
                for edition in FREEBSD_EDITIONS {
                    let iso = format!("FreeBSD-{release}-RELEASE-arm64-aarch64-{edition}.iso.xz");
                    let checksum = find_checksum(checksums.as_deref(), &iso);
                    let url = format!("{FREEBSD_AARCH64_RELEASES}ISO-IMAGES/{release}/{iso}");
                    releases.push(Config {
                        guest_os: GuestOS::FreeBSD,
                        arch: Arch::aarch64,
                        iso: Some(vec![Source::Web(WebSource::new(url, checksum, Some(ArchiveFormat::Xz), None))]),
                        ..Default::default()
                    });
                }
            }
        } else {
            log::warn!("Failed to fetch FreeBSD aarch64 releases");
        }
        releases
    }
}
