use crate::config::*;
use crate::validate;
use anyhow::{Result, anyhow, bail};
use std::convert::TryFrom;
use std::net::{TcpListener, SocketAddrV4, Ipv4Addr};
use core::num::NonZeroUsize;
use std::path::{Path, PathBuf};

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
        Some(core_string) => core_string.parse::<NonZeroUsize>()?.get(),
        None => match logical {
            _ if physical > logical => bail!("Found more physical cores than logical cores. Please manually set your core count in the configuration file."),
            32.. => 16,
            16.. => 8,
            8.. => 4,
            4.. => 2,
            _ => 1,
        },
    }, logical > physical))
}

impl TryFrom<(Option<String>, Option<String>)> for BootType {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<String>)) -> Result<Self> {
        let secure_boot = parse_optional_bool(value.1, false)?;
        Ok(match value.0 {
            Some(boot_type) => match boot_type.as_str() {
                "efi" => Self::Efi { secure_boot },
                _ if secure_boot => bail!("Secure boot is only supported with the EFI boot type."),
                "legacy" | "bios" => Self::Legacy,
                _ => bail!("Specified boot type {} is invalid. Please check your config file. Valid boot types are 'efi', 'legacy'/'bios'", boot_type),
            },
            _ => Self::Efi { secure_boot },
        })
    }
}

pub fn parse_optional_bool(value: Option<String>, default: bool) -> Result<bool> {
    match value {
        Some(text) => match text.as_str() {
            "true" | "on" => Ok(true),
            "false" | "off" => Ok(false),
            _ => bail!("Invalid value: {}", text),
        },
        None => Ok(default),
    }
}

pub const BYTES_PER_GB: u64 = 1024 * 1024 * 1024;
pub fn size_unit(input: Option<String>, ram: Option<u64>) -> Result<Option<u64>> {
    Ok(match input {
        Some(size) => Some({
            let unit_size = match size.chars().last().unwrap() {
                'K' => 1024.0,
                'M' => 1024.0 * 1024.0,
                'G' => BYTES_PER_GB as f64,
                'T' => 1024.0 * BYTES_PER_GB as f64,
                _ => bail!("Invalid size: {}", size),
            };
            match size[..size.len()-1].parse::<f64>() {
                Ok(size) => (size * unit_size) as u64,
                Err(_) => bail!("Invalid size: {}", size),
            }
        }),
        None => ram.map(|ram| match ram / (1000 * 1000 * 1000) {
            128.. => 32 * BYTES_PER_GB,
            64.. => 16 * BYTES_PER_GB,
            16.. => 8 * BYTES_PER_GB,
            8.. => 4 * BYTES_PER_GB,
            _ => ram,
        }),
    })
}

impl TryFrom<(Option<String>, Option<Keyboard>)> for Keyboard {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<Keyboard>)) -> Result<Self> {
        Ok(match value {
            (_, Some(kbtype)) => kbtype,
            (Some(kbtype), _) => match kbtype.as_str() {
                "usb" => Self::Usb,
                "ps2" => Self::PS2,
                "virtio" => Self::Virtio,
                _ => bail!("Invalid keyboard type: {}", kbtype),
            },
            _ => Self::Usb,
        })
    }
}

impl TryFrom<(Option<String>, Option<Display>)> for Display {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<Display>)) -> Result<Display> {
        Ok(match value {
            (_, Some(display)) => display,
            (Some(display), _) => match display.as_str() {
                "sdl" => Display::Sdl,
                "gtk" => Display::Gtk,
                "spice" => Display::Spice,
                "spice-app" => Display::SpiceApp,
                _ => bail!("Invalid display type: {}", display),
            },
            _ => Display::Sdl,
        })
    }
}

impl TryFrom<(&Path, Option<String>, Option<String>)> for Image {
    type Error = anyhow::Error;
    fn try_from(value: (&Path, Option<String>, Option<String>)) -> Result<Self> {
        let file_path= |file: String, filetype: &str| {
            let full_path = value.0.join(&file);
            let path = file.parse::<PathBuf>().map_err(|_| anyhow!("Could not parse {} file path: {}", filetype, file))?;
            if path.exists() {
                Ok(path)
            } else if full_path.exists() {
                Ok(full_path.relativize()?)
            } else {
                bail!("{} file does not exist: {}", filetype, file);
            }
        };
        Ok(match value {
            (_, Some(_), Some(_)) => bail!("Config file cannot contain both an img and an iso file."),
            (_, Some(iso), _) => Self::Iso(file_path(iso, "ISO")?),
            (.., Some(img)) => Self::Img(file_path(img, "IMG")?),
            _ => Self::None,
        })
    }
}

impl TryFrom<&Vec<String>> for Snapshot {
    type Error = anyhow::Error;
    fn try_from(input: &Vec<String>) -> Result<Self> {
        Ok(match input[0].as_str() {
            "apply" if input.len() == 2 => Self::Apply(input[1].clone()),
            "create" if input.len() == 2 => Self::Create(input[1].clone()),
            "delete" if input.len() == 2 => Self::Delete(input[1].clone()),
            "info" if input.len() == 1 => Self::Info,
            _ => bail!("Invalid parameters to argument --snapshot: {}", input.join(" ")),
        })
    }
}

impl TryFrom<(Option<String>, Option<String>)> for Network {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<String>)) -> Result<Self> {
        Ok(match value {
            (Some(network_type), mac_addr) => match network_type.to_lowercase().as_str() {
                "restrict" | "nat" | "none" if mac_addr.is_some() => bail!("MAC Addresses are only supported for bridged networking."),
                "restrict" => Network::Restrict,
                "nat" => Network::Nat,
                "none" => Network::None,
                bridge => Network::Bridged { bridge: bridge.to_string(), mac_addr }
            },
            _ => Network::Nat,
        })
    }
}

pub fn port_forwards(bash_array: Option<String>) -> Result<Option<Vec<(u16, u16)>>> {
    match bash_array {
        Some(array) => {
            let ports = array.split_whitespace().filter_map(|pair| pair.trim_matches(['(', ')', ',', ' ', '"']).split_once(':'));
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

impl TryFrom<(Option<String>, Option<String>)> for GuestOS {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<String>)) -> Result<Self> {
        match value {
            (Some(os), macos_release) => Ok(match os.to_lowercase().as_str() {
                "macos" => Self::MacOS(MacOSRelease::try_from(macos_release)?),
                _ if macos_release.is_some() => bail!("macOS releases are not supported for OS {}", os),
                "linux" => Self::Linux,
                "linux_old" => Self::LinuxOld,
                "windows" => Self::Windows,
                "windows-server" => Self::WindowsServer,
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
            _ => bail!("The configuration file must contain a guest_os field"),
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

fn find_monitor(monitor: &str, host1: Option<String>, port1: Option<u16>, host2: Option<String>, port2: Option<u16>, socketpath: PathBuf) -> Result<Monitor> {
    match monitor {
        "none" => if host1.is_some() || host2.is_some() || port2.is_some() {
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



impl TryFrom<([(Option<String>, Option<String>, Option<u16>); 2], PathBuf)> for Monitor {
    type Error = anyhow::Error;
    fn try_from(value: ([(Option<String>, Option<String>, Option<u16>); 2], PathBuf)) -> Result<Self> {
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
                "usb" => Mouse::Usb,
                "ps2" => Mouse::PS2,
                "virtio" => Mouse::Virtio,
                _ => bail!("Invalid mouse type: {}", mouse),
            },
            (_, _, os) => match os {
                GuestOS::FreeBSD | GuestOS::GhostBSD => Mouse::Usb,
                _ => Mouse::Tablet,
            },
        })
    }
}

impl TryFrom<Option<String>> for MacOSRelease {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Self> {
        Ok(match value {
            Some(release) => match release.as_str() {
                "high-sierra" => MacOSRelease::HighSierra,
                "mojave" => MacOSRelease::Mojave,
                "catalina" => MacOSRelease::Catalina,
                "big-sur" => MacOSRelease::BigSur,
                "monterey" => MacOSRelease::Monterey,
                "ventura" => MacOSRelease::Ventura,
                "sonoma" => MacOSRelease::Sonoma,
                _ => bail!("Unsupported macOS release: {}", release),
            },
            _ => bail!("Your configuration file must include a macOS release."),
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

impl TryFrom<(Option<String>, Option<USBController>, &GuestOS)> for USBController {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<USBController>, &GuestOS)) -> Result<Self> {
        Ok(match value {
            (_, Some(controller), _) => controller,
            (Some(controller), ..) => match controller.as_str() {
                "none" => Self::None,
                "ehci" => Self::Ehci,
                "xhci" => Self::Xhci,
                _ => bail!("Invalid USB controller: {}", controller),
            },
            (.., GuestOS::Solaris) => Self::Xhci,
            (.., GuestOS::MacOS(release)) if release >= &MacOSRelease::BigSur => Self::Xhci,
            _ => Self::Ehci,
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
    input.map(|devices| devices.split_whitespace().map(|device| device.trim_matches(['(', ')', ',', ' ', '"']).to_string()).collect())
}

pub fn parse_optional_path(value: Option<String>, name: &str, vm_dir: &Path) -> Result<Option<PathBuf>> {
    Ok(match value {
        Some(path_string) => {
            let path = path_string.parse::<PathBuf>().map_err(|_| anyhow!("Could not parse {} path: {}", name, path_string))?;
            let absolute_path = vm_dir.join(&path);
            log::debug!("Path: {:?} {}, Absolute: {:?} {}, name: {}", path, path.exists(), absolute_path, absolute_path.exists(), name);
            if path.exists() {
                Some(path)
            } else if absolute_path.exists() {
                Some(absolute_path)
            } else {
                bail!("Could not find {} file: {}. Please verify that it exists.", name, path_string);
            }
        },
        None => None,
    })
}

pub trait Relativize {
    fn relativize(&self) -> Result<PathBuf>;
}
impl Relativize for PathBuf {
    fn relativize(&self) -> Result<PathBuf> {
        log::debug!("Relativizing path: {:?}", self);
        let current_dir = std::env::current_dir()?;
        Ok(pathdiff::diff_paths(self, current_dir)
            .unwrap_or(self.clone()))
    }
}
