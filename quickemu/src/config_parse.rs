use crate::config::*;
use crate::validate;
use anyhow::{Result, anyhow, bail};
use std::convert::TryFrom;
use core::num::NonZeroUsize;
use std::path::PathBuf;

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

pub fn cpu_cores(input: Option<NonZeroUsize>, logical: usize, physical: usize) -> Result<(usize, bool)> {
    Ok((match input {
        Some(cores) => cores.into(),
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

pub const BYTES_PER_GB: u64 = 1024 * 1024 * 1024;
pub fn default_ram(system_ram: u64) -> u64 {
    match system_ram / (1000 * 1000 * 1000) {
        128.. => 32 * BYTES_PER_GB,
        64.. => 16 * BYTES_PER_GB,
        16.. => 8 * BYTES_PER_GB,
        8.. => 4 * BYTES_PER_GB,
        _ => system_ram
    }
}
pub fn size_unit(size: &str) -> Result<u64> {
    let unit_size = match size.chars().last().unwrap() {
        'K' => 1024.0,
        'M' => 1024.0 * 1024.0,
        'G' => BYTES_PER_GB as f64,
        'T' => 1024.0 * BYTES_PER_GB as f64,
        _ => bail!("Invalid size (unit): {}", size),
    };
    match size[..size.len()-1].parse::<f64>() {
        Ok(size) => Ok((size * unit_size) as u64),
        Err(_) => bail!("Invalid size (integer): {}", size),
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

impl TryFrom<(SerdeMonitor, Option<String>, Option<String>, Option<u16>, u16, PathBuf)> for Monitor {
    type Error = anyhow::Error;
    fn try_from(value: (SerdeMonitor, Option<String>, Option<String>, Option<u16>, u16, PathBuf)) -> Result<Self> {
        let monitor_type = value.1.unwrap_or(value.0.r#type);
        let host = value.2.or(value.0.telnet_host);
        let port = value.3.or(value.0.telnet_port).unwrap_or(value.4);
        let socketpath = value.5;

        match monitor_type.as_str() {
            "none" if host.is_some() || value.3.is_some() => bail!("Monitor type 'none' cannot have any additional parameters."),
            "none" => Ok(Monitor::None),
            "telnet" => Ok(Monitor::Telnet { address: (host.unwrap_or("127.0.0.1".to_string()) + ":" + &port.to_string()).parse()? }),
            "socket" => Ok(Monitor::Socket { socketpath }),
            _ => bail!("Invalid monitor type: {}", monitor_type),
        }
    }
}

impl From<&GuestOS> for Mouse {
    fn from(value: &GuestOS) -> Self {
        match value {
            GuestOS::FreeBSD | GuestOS::GhostBSD => Self::Usb,
            _ => Self::Tablet,
        }
    }
}

impl From<(Resolution, Option<u32>, Option<u32>, Option<String>)> for Resolution {
    fn from(value: (Resolution, Option<u32>, Option<u32>, Option<String>)) -> Self {
        match value {
            (_, Some(width), Some(height), _) => Self::Custom { width, height },
            (.., Some(screen)) => Self::Display(screen),
            (res, ..) => res,
        }
    }
}

impl From<&GuestOS> for USBController {
    fn from(value: &GuestOS) -> Self {
        match value {
            GuestOS::Solaris => Self::Xhci,
            GuestOS::MacOS { release } if release >= &MacOSRelease::BigSur => Self::Xhci,
            _ => Self::Ehci,
        }
    }
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


// Below are implementations for the legacy config file format, used for the migrate_config option.

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

impl TryFrom<Option<String>> for Arch {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Self> {
        Ok(match value {
            Some(arch) => match arch.as_str() {
                "x86_64" => Self::x86_64,
                "aarch64" => Self::aarch64,
                "riscv64" => Self::riscv64,
                _ => bail!("{} is not a supported architecture. Please check your legacy config file.", arch),
            }
            None => Default::default(),
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
            None => Default::default(),
        })
    }
}

impl TryFrom<Option<String>> for Display {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Display> {
        Ok(match value {
            Some(display) => match display.as_str() {
                "sdl" => Display::Sdl,
                "gtk" => Display::Gtk,
                #[cfg(not(target_os = "macos"))]
                "spice" => Display::Spice,
                #[cfg(not(target_os = "macos"))]
                "spice-app" => Display::SpiceApp,
                #[cfg(target_os = "macos")]
                "cocoa" => Display::Cocoa,
                _ => bail!("Invalid display type: {}", display),
            },
            _ => Default::default(),
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

impl TryFrom<Option<String>> for Keyboard {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Self> {
        Ok(match value {
            Some(kbtype) => match kbtype.as_str() {
                "usb" => Self::Usb,
                "ps2" => Self::PS2,
                "virtio" => Self::Virtio,
                _ => bail!("Invalid keyboard type: {}", kbtype),
            },
            _ => Default::default(),
        })
    }
}

impl TryFrom<(Option<String>, Option<String>, Option<String>)> for SerdeMonitor {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<String>, Option<String>)) -> Result<Self> {
        let monitor_type = value.0.unwrap_or("socket".to_string());
        let telnet_host = value.1;
        let telnet_port = value.2.map(|port| port.parse::<u16>()
            .map_err(|_| anyhow!("Invalid port number: {}", port))).transpose()?;
        Ok(Self { r#type: monitor_type, telnet_host, telnet_port })
    }
}

impl TryFrom<Option<String>> for SoundCard {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Self> {
        Ok(match value {
            Some(card) => match card.as_str() {
                "none" => SoundCard::None,
                "ac97" => SoundCard::AC97,
                "es1370" => SoundCard::ES1370,
                "sb16" => SoundCard::SB16,
                "intel-hda" => SoundCard::IntelHDA,
                _ => bail!("Invalid sound card: {}", card),
            },
            _ => Default::default(),
        })
    }
}

impl TryFrom<Option<String>> for Resolution {
    type Error = anyhow::Error;
    fn try_from(value: Option<String>) -> Result<Self> {
        Ok(match value {
            Some(res) => {
                let (w, h) = res.split_once('x').ok_or_else(|| anyhow!("Invalid resolution: {}", res))?;
                Self::Custom { width: w.parse()?, height: h.parse()? }
            },
            _ => Default::default(),
        })
    }
}

impl TryFrom<(Option<String>, Option<String>)> for GuestOS {
    type Error = anyhow::Error;
    fn try_from(value: (Option<String>, Option<String>)) -> Result<Self> {
        match value {
            (Some(os), macos_release) => Ok(match os.to_lowercase().as_str() {
                "macos" => Self::MacOS { release: MacOSRelease::try_from(macos_release)? },
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
