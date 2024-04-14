use crate::config::*;
use crate::validate;
use anyhow::{Result, anyhow, bail};
use std::convert::TryFrom;
use std::net::{TcpListener, SocketAddrV4, Ipv4Addr};

impl From<Option<String>> for Access {
    fn from(value: Option<String>) -> Self {
        match value {
            Some(input) => match input.as_str() {
                "remote" => Self::Remote,
                "local" => Self::Local,
                _ => Self::Address(input),
            },
            None => Self::Local,
        }
    }
}

impl TryFrom<Option<String>> for Arch {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Self> {
        Ok(match value {
            Some(arch) => match arch.as_str() {
                "x86_64" => Self::x86_64,
                "aarch64" => Self::aarch64,
                "riscv64" => Self::riscv64,
                _ => bail!("{} is not a supported architecture. Please check your config file.", arch),
            }
            None => Self::x86_64,
        })
    }
}
    

pub fn cpu_cores(input: Option<String>, logical: usize, physical: usize) -> Result<(usize, bool)> {
    Ok((match input {
        Some(core_string) => core_string.parse::<usize>()?,
        None => match logical {
            32.. => 16,
            16.. => 8,
            8.. => 4,
            4.. => 2,
            _ => 1,
        },
    }, logical != physical))
}

impl TryFrom<Option<String>> for BootType {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Self> {
        Ok(match value {
            Some(boot_type) => match boot_type.as_str() {
                "efi" => Self::EFI,
                "legacy" | "bios" => Self::Legacy,
                _ => bail!("Specified boot type {} is invalid. Please check your config file", boot_type),
            },
            None => Self::EFI,
        })
    }
}

pub fn parse_optional_bool(value: Option<String>) -> Result<Option<bool>> {
    match value {
        Some(text) => match text.as_str() {
            "true" => Ok(Some(true)),
            "false" => Ok(Some(false)),
            _ => bail!("Invalid boolean: {}", text),
        },
        None => Ok(None),
    }
}

pub fn size_unit(input: Option<String>, ram: Option<u64>) -> Result<Option<u64>> {
    Ok(match input {
        Some(size) => Some({
            let unit_size = match size.chars().last().unwrap() {
                'K' => 1024,
                'M' => 1024 * 1024,
                'G' => 1024 * 1024 * 1024,
                'T' => 1024 * 1024 * 1024 * 1024,
                _ => bail!("Invalid size: {}", size),
            };
            match size[..size.len()-1].parse::<u64>() {
                Ok(size) => size*unit_size,
                Err(_) => bail!("Invalid size: {}", size),
            }
        }),
        None => match ram {
            Some(ram) => Some(match ram / (1000 * 1000 * 1000) {
                128.. => 32 * (1024 * 1024 * 1024),
                64.. => 16 * (1024 * 1024 * 1024),
                16.. => 8 * (1024 * 1024 * 1024),
                8.. => 4 * (1024 * 1024 * 1024),
                _ => ram,
            }),
            None => None,
        }
    })
}

impl TryFrom<(Option<String>, Option<Keyboard>)> for Keyboard {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<Keyboard>)) -> Result<Self> {
        Ok(match value {
            (_, Some(kbtype)) => kbtype,
            (Some(kbtype), _) => match kbtype.as_str() {
                "usb" => Self::USB,
                "ps2" => Self::PS2,
                "virtio" => Self::Virtio,
                _ => bail!("Invalid keyboard type: {}", kbtype),
            },
            _ => Self::USB,
        })
    }
}

impl TryFrom<(Option<String>, Option<Display>)> for Display {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<Display>)) -> Result<Display> {
        Ok(match value {
            (_, Some(display)) => display,
            (Some(display), _) => match display.as_str() {
                "sdl" => Display::SDL,
                "gtk" => Display::GTK,
                "spice" => Display::Spice,
                "spice-app" => Display::SpiceApp,
                _ => bail!("Invalid display type: {}", display),
            },
            _ => Display::SDL,
        })
    }
}

pub fn image(iso: Option<String>, img: Option<String>) -> Image {
    if iso.is_some() && img.is_some() {
        log::error!("Config file cannot contain both an img and an iso file.");
        std::process::exit(1);
    }
    if iso.is_some() {
        Image::ISO(iso.unwrap())
    } else if img.is_some() {
        Image::IMG(img.unwrap())
    } else {
        Image::None
    }
}

const SNAPSHOT_TYPES: [&str; 4] = ["apply", "create", "delete", "info"];

pub fn snapshot(input: Option<Vec<String>>) -> Option<Snapshot> {
    match input {
        Some(input) if SNAPSHOT_TYPES.contains(&input[0].as_str()) => match input[0].as_str() {
            "apply" if input.len() == 2 => Some(Snapshot::Apply(input[1].clone())),
            "create" if input.len() == 2 => Some(Snapshot::Create(input[1].clone())),
            "delete" if input.len() == 2 => Some(Snapshot::Delete(input[1].clone())),
            "info" if input.len() == 1 => Some(Snapshot::Info),
            _ => unimplemented!(),
        },
        Some(invalid) => {
            log::error!("Argument '--snapshot {}' is not supported.", invalid[0]);
            std::process::exit(1);
        },
        None => None
    }
}

impl TryFrom<(Option<String>, Option<String>)> for Network {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<String>)) -> Result<Self> {
        Ok(match value {
            (Some(network_type), macaddr) => match network_type.to_lowercase().as_str() {
                "bridged" | "br0" => Network::Bridged { mac_addr: macaddr },
                _ if macaddr.is_some() => bail!("MAC address is only supported for the bridged network type."),
                "restrict" => Network::Restrict,
                "nat" => Network::NAT,
                "none" => Network::None,
                _ => bail!("Network type {} is not supported.", network_type),
            },
            _ => Network::NAT,
        })
    }
}

pub fn port_forwards(bash_array: Option<String>) -> Result<Option<Vec<(u16, u16)>>> {
    match bash_array {
        Some(array) => {
            let ports = array.split_whitespace().filter_map(|pair| pair.trim_matches([',', ' ', '"']).split_once(':'));
            ports.map(|(host, guest)| {
                Ok(Some((host.parse::<u16>()?, guest.parse::<u16>()?)))
            }).collect()
        },
        None => Ok(None),
    }
}

impl TryFrom<Option<String>> for PreAlloc {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Self> {
        Ok(match value {
            Some(variant) => match variant.as_str() {
                "off" => PreAlloc::Off,
                "metadata" => PreAlloc::Metadata,
                "falloc" => PreAlloc::Falloc,
                "full" => PreAlloc::Full,
                _ => bail!("Invalid preallocation type."),
            },
            None => PreAlloc::Off,
        })
    }
}

impl From<(Option<String>, Option<String>)> for PublicDir {
    fn from(value: (Option<String>, Option<String>)) -> Self {
        match value {
            (_, Some(dirtype)) | (Some(dirtype), _) => match dirtype.as_str() {
                "default" => PublicDir::Default,
                "none" => PublicDir::None,
                _ => PublicDir::Custom(dirtype),
            },
            _ => PublicDir::Default,
        }
    }
}

impl TryFrom<Option<String>> for GuestOS {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Self> {
        match value {
            Some(os) => Ok(match os.to_lowercase().as_str() {
                "linux" => Self::Linux,
                "windows" => Self::Windows,
                "windows-server" => Self::WindowsServer,
                "macos" => Self::MacOS,
                "freebsd" => Self::FreeBSD,
                "ghostbsd" => Self::GhostBSD,
                "freedos" => Self::FreeDOS,
                "haiku" => Self::Haiku,
                "solaris" => Self::Solaris,
                "kolibrios" => Self::KolibriOS,
                "reactos" => Self::ReactOS,
                "batocera" => Self::Batocera,
                _ => bail!("The guest_os specified in the configuration file is unsupported."),
            }),
            None => bail!("The configuration file must contain a guest_os field"),
        }
    }
}

pub fn keyboard_layout(value: (Option<String>, Option<String>)) -> Result<Option<String>> {
    Ok(match value {
        (_, Some(layout)) => Some(validate::validate_keyboard_layout(layout)?),
        (Some(layout), _) => Some(validate::validate_keyboard_layout(layout)?),
        _ => match std::env::consts::OS {
            "macos" => Some("en-us".to_string()),
            _ => None,
        },
    })
}

fn find_monitor(monitor: &str, host1: Option<String>, port1: Option<u16>, host2: Option<String>, port2: Option<u16>, socketpath: std::path::PathBuf) -> Result<Monitor> {
    match monitor {
        "none" => if host1.is_some() || port1.is_some() || host2.is_some() || port2.is_some() {
            bail!("Monitor type 'none' cannot have any additional parameters.")
        } else {
            Ok(Monitor::None)
        },
        "telnet" => Ok(Monitor::Telnet {
            host: match (host1, host2) {
                (_, Some(host)) => host,
                (Some(host), _) => host,
                _ => "localhost".to_string(),
            },
            port: port2.unwrap_or(port1.unwrap()),
        }),
        "socket" => Ok(Monitor::Socket { socketpath }),
        _ => bail!("Invalid monitor type: {}", monitor),
    }
}



impl TryFrom<([(Option<String>, Option<String>, Option<u16>); 2], std::path::PathBuf)> for Monitor {
    type Error = anyhow::Error;
    fn try_from(value: ([(Option<String>, Option<String>, Option<u16>); 2], std::path::PathBuf)) -> Result<Self> {
        let (socketpath, host1, port1, host2, port2) = (value.1, value.0[0].1.clone(), value.0[0].2, value.0[1].1.clone(), value.0[1].2);
        match (&value.0[0].0, &value.0[1].0) {
            (_, Some(monitor)) => find_monitor(monitor, host1, port1, host2, port2, socketpath),
            (Some(monitor), _) => find_monitor(monitor, host1, port1, host2, port2, socketpath),
            _ => find_monitor("socket", host1, port1, host2, port2, socketpath),
        }
    }
}

impl TryFrom<(Option<String>, Option<Mouse>, &GuestOS)> for Mouse {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<Mouse>, &GuestOS)) -> Result<Self> {
        Ok(match value {
            (_, Some(mouse), _) => mouse,
            (Some(mouse), ..) => match mouse.as_str() {
                "usb" => Mouse::USB,
                "ps2" => Mouse::PS2,
                "virtio" => Mouse::Virtio,
                _ => bail!("Invalid mouse type: {}", mouse),
            },
            (_, _, os) => match os {
                GuestOS::FreeBSD | GuestOS::GhostBSD => Mouse::USB,
                _ => Mouse::Tablet,
            },
        })
    }
}

impl TryFrom<(Option<String>, &GuestOS)> for MacOSRelease {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, &GuestOS)) -> Result<Self> {
        Ok(match value {
            (Some(release), GuestOS::MacOS) => match release.as_str() {
                "high-sierra" => MacOSRelease::HighSierra,
                "mojave" => MacOSRelease::Mojave,
                "catalina" => MacOSRelease::Catalina,
                "big-sur" => MacOSRelease::BigSur,
                "monterey" => MacOSRelease::Monterey,
                "ventura" => MacOSRelease::Ventura,
                "sonoma" => MacOSRelease::Sonoma,
                _ => bail!("Unsupported macOS release: {}", release),
            },
            (Some(_), guest_os) => bail!("macOS releases are not supported for OS {}", guest_os),
            (_, GuestOS::MacOS) => bail!("Your configuration file must include a macOS release."),
            _ => MacOSRelease::None,
        })
    }
}

impl TryFrom<(Option<String>, Option<String>, Option<String>)> for Resolution {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<String>, Option<String>)) -> Result<Self> {
        match value {
            (Some(resolution), ..) | (.., Some(resolution)) => {
                let (width, height) = resolution.split_once('x').ok_or_else(|| anyhow!("Invalid resolution: {}", resolution))?;
                Ok(Resolution::Custom {
                    width: width.parse()?,
                    height: height.parse()?,
                })
            },
            (_, Some(screen), _) => Ok(Resolution::Display(screen)),
            _ => Ok(Resolution::Default),
        }
    }
}

impl TryFrom<(Option<String>, Option<USBController>)> for USBController {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<USBController>)) -> Result<Self> {
        Ok(match value {
            (_, Some(controller)) => controller,
            (Some(controller), _) => match controller.as_str() {
                "none" => USBController::None,
                "ehci" => USBController::EHCI,
                "xhci" => USBController::XHCI,
                _ => bail!("Invalid USB controller: {}", controller),
            },
            _ => USBController::EHCI,
        })
    }
}

impl TryFrom<(Option<String>, Option<SoundCard>)> for SoundCard {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<SoundCard>)) -> Result<Self> {
        Ok(match value {
            (_, Some(card)) => card,
            (Some(card), _) => match card.as_str() {
                "none" => SoundCard::None,
                "ac97" => SoundCard::AC97,
                "es1370" => SoundCard::ES1370,
                "sb16" => SoundCard::SB16,
                "intel-hda" => SoundCard::IntelHDA,
                _ => bail!("Invalid sound card: {}", card),
            },
            _ => SoundCard::IntelHDA,
        })
    }
}

pub fn port(input: (Option<String>, Option<u16>), default: u16, offset: u16) -> Result<u16> {
    Ok(match input {
        (_, Some(port)) => port,
        (Some(port), _) => port.parse()?,
        _ => (default..=default+offset).find(|port| {
            let port = SocketAddrV4::new(Ipv4Addr::LOCALHOST, *port);
            TcpListener::bind(port).is_ok()
        }).ok_or_else(|| anyhow!("Unable to find a free port in range {}-{}", default, default+offset))?,
    })
}

pub fn usb_devices(input: Option<String>) -> Option<Vec<String>> {
    match input {
        Some(devices) => Some(devices.split_whitespace().map(|device| device.trim_matches(['"', '(']).to_string()).collect()),
        None => None,
    }
}
