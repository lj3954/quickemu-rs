use anyhow::{anyhow, Result};
use std::ffi::OsString;
use crate::config::{Arch, BooleanDisplay, Display, Monitor, SoundCard, GuestOS, Resolution};
use crate::qemu_args::find_port;

impl SoundCard {
    pub fn to_args(&self) -> (Vec<String>, Option<Vec<String>>) {
        match self {
            Self::None => (vec![], Some(vec!["Sound: Disabled".into()])),
            Self::AC97 => (vec!["-device".into(), "ac97,audiodev=audio0".into()], Some(vec!["Emulated sound card: AC97".into()])),
            Self::ES1370 => (vec!["-device".into(), "es1370,audiodev=audio0".into()], Some(vec!["Emulated sound card: ES1370".into()])),
            Self::SB16 => (vec!["-device".into(), "sb16,audiodev=audio0".into()], Some(vec!["Emulated sound card: Sound Blaster 16".into()])),
            Self::IntelHDA => (vec!["-device".into(), "intel-hda".into(), "-device".into(), "hda-duplex,audiodev=audio0".into()], Some(vec!["Emulated sound card: Intel HDA".into()])),
        }
    }
}

impl Display {
    pub fn audio_arg(&self) -> [OsString; 2] {
        match self {
            #[cfg(target_os = "linux")]
            Self::None | Self::Spice | Self::SpiceApp => ["-audiodev".into(), "spice,id=audio0".into()],
            #[cfg(target_os = "macos")]
            _ => ["-audiodev".into(), "coreaudio,id=audio0".into()],
            #[cfg(not(target_os = "macos"))]
            _ => ["-audiodev".into(), "pa,id=audio0".into()],
        }
    }

    pub fn display_args(&self, guest_os: &GuestOS, arch: &Arch, resolution: Resolution, screenpct: Option<u32>, accel: bool) -> Result<(Vec<String>, Option<Vec<String>>)> {
        let virtio_vga = || if accel { "virtio-vga-gl" } else { "virtio-vga" };
        let (display_device, friendly_display_device) = match arch {
            Arch::x86_64 => match guest_os {
                GuestOS::Linux => match self {
                    Self::None => ("virtio-gpu", "VirtIO GPU"),
                    #[cfg(target_os = "linux")]
                    Self::Spice | Self::SpiceApp => ("virtio-gpu", "VirtIO GPU"),
                    _ => (virtio_vga(), "VirtIO VGA"),
                },
                GuestOS::Windows | GuestOS::WindowsServer if self == &Self::Sdl => (virtio_vga(), "VirtIO VGA"),
                #[cfg(target_os = "linux")]
                GuestOS::Windows | GuestOS::WindowsServer if self == &Self::SpiceApp => (virtio_vga(), "VirtIO VGA"),
                #[cfg(target_os = "macos")]
                GuestOS::Windows | GuestOS::WindowsServer if self == &Self::Cocoa => ("virtio-vga", "VirtIO VGA"),
                GuestOS::Solaris | GuestOS::LinuxOld => ("vmware-svga,vgamem_mb=256", "VMware SVGA"),
                _ => ("qxl-vga,ram_size=65536,vram_size=65536,vgamem_mb=64", "QXL"),
            },
            Arch::riscv64 => (virtio_vga(), "VirtIO VGA"),
            Arch::aarch64 => ("virtio-gpu", "VirtIO GPU"),
        };
        let gl = if accel { "on" } else { "off" };

        let display_render = match self {
            Self::Gtk => "gtk,grab-on-hover=on,zoom-to-fit=off,gl=".to_string() + gl,
            Self::None => "none".to_string(),
            Self::Sdl => "sdl,gl=".to_string() + gl,
            #[cfg(target_os = "linux")]
            Self::Spice => "none".to_string(),
            #[cfg(target_os = "linux")]
            Self::SpiceApp => "spice-app,gl=".to_string() + gl,
            #[cfg(target_os = "macos")]
            Self::Cocoa => "cocoa".to_string(),
        };

        let mut message = format!("Display: {}, Device: {}, GL: {}, VirGL: {}", self, friendly_display_device, accel.as_str(), (display_device == "virtio-vga-gl").as_str());

        let video = if matches!(guest_os, GuestOS::LinuxOld | GuestOS::Solaris) {
            display_device.to_string()
        } else {
            let (width, height) = match resolution {
                Resolution::Custom { width, height } => (width, height),
                Resolution::Display(display) => display_resolution(Some(display), screenpct)?,
                Resolution::Default => display_resolution(None, screenpct)?,
            };
            message.push_str(&format!(", Resolution: {width}x{height}"));
            format!("{display_device},xres={width},yres={height}")
        };

        Ok((vec!["-display".into(), display_render, "-device".into(), video, "-vga".into(), "none".into()], Some(vec![message])))
    }
}

fn display_resolution(name: Option<String>, screenpct: Option<u32>) -> Result<(u32, u32)> {
    let display_info = display_info::DisplayInfo::all()?;
    log::debug!("Displays: {:?}", display_info);
    let display = if let Some(monitor) = name {
        display_info.iter().find(|available| available.name == monitor).ok_or_else(|| anyhow!("Could not find a display matching the name {}", monitor))?
    } else {
        display_info.iter().find(|available| available.is_primary)
            .unwrap_or(display_info.first().ok_or_else(|| anyhow!("Could not find a monitor. Please manually specify the resolution in your config file."))?)
    };

    let (width, height) = match display.width {
        _ if screenpct.is_some() => ((screenpct.unwrap() * display.width) / 100, (screenpct.unwrap() * display.height) / 100),
        3840.. => (3200, 1800),
        2560.. => (2048, 1152),
        1920.. => (1664, 936),
        1280.. => (1152, 648),
        _ => (display.width, display.height),
    };

    Ok((width, height))
}

impl Monitor {
    pub fn to_args(&self, variant: &str) -> Result<(Vec<OsString>, Option<Vec<String>>)> {
        let mut arg = OsString::from("-");
        let text = variant[0..1].to_uppercase() + &variant[1..] + ": ";
        arg.push(variant);
        Ok(match self {
            Self::None => (vec![arg, "none".into()], Some(vec![text + "None"])),
            Self::Telnet { port, host } => {
                let mut telnet = OsString::from("telnet:");
                telnet.push(host);
                telnet.push(":");
                telnet.push(find_port(*port, 9)
                    .ok_or_else(|| anyhow!("Could not find an open telnet port between {} and {}", port, port + 9))?
                    .to_string());
                telnet.push(",server,nowait");
                (vec![arg, telnet], Some(vec![text + "On host: telnet " + host + " " + &port.to_string()]))
            },
            Self::Socket { socketpath } => {
                let mut socket = OsString::from("unix:");
                socket.push(socketpath);
                socket.push(",server,nowait");
                (vec![arg, socket], Some(vec![text + "On host: " + &socketpath.to_string_lossy()]))
            },
        })
    }
}
