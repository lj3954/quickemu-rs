use anyhow::{anyhow, Result};
use std::ffi::OsString;
use crate::config::{BooleanDisplay, Display, SoundCard, GuestOS, Resolution};

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
            Self::None | Self::Spice | Self::SpiceApp => ["-audiodev".into(), "spice,id=audio0".into()],
            _ => ["-audiodev".into(), "pa,id=audio0".into()],
        }
    }

    pub fn display_args(&self, guest_os: &GuestOS, resolution: Resolution, accel: bool) -> Result<(Vec<String>, Option<Vec<String>>)> {
        let display_device = match guest_os {
            GuestOS::Linux => match self {
                Self::None | Self::Spice | Self::SpiceApp => "virtio-gpu",
                _ => "virtio-vga-gl",
            },
            GuestOS::MacOS(_) => "qxl-vga,ram_size=65536,vram_size=65536,vgamem_mb=64",
            GuestOS::Windows | GuestOS::WindowsServer => match self {
                Self::Sdl | Self::SpiceApp => "virtio-vga-gl",
                _ => "qxl-vga,ram_size=65536,vram_size=65536,vgamem_mb=64",
            },
            GuestOS::Solaris => "vmware-svga",
            _ => "qxl-vga,ram_size=65536,vram_size=65536,vgamem_mb=64",
        };
        let gl = if accel { "on" } else { "off" };

        let display_render = match self {
            Self::Gtk => "gtk,grab-on-hover=on,zoom-to-fit=off,gl=".to_string() + gl,
            Self::None | Self::Spice => "none".to_string(),
            Self::Sdl => "sdl,gl=".to_string() + gl,
            Self::SpiceApp => "spice-app,gl=".to_string() + gl,
        };

        let video = match (resolution, guest_os) {
            (Resolution::Custom { width, height }, _) => format!("{display_device},xres={width},yres={height}"),
            (Resolution::Display(display), _) => {
                let (width, height) = display_resolution(Some(display))?;
                format!("{display_device},xres={width},yres={height}")
            },
            (Resolution::Default, GuestOS::Linux) => {
                let (width, height) = display_resolution(None)?;
                format!("{display_device},xres={width},yres={height}")
            },
            _ => display_device.to_string(),
        };

        let message = format!("Display: {}, {}, GL: {}, VirGL: {}", self, display_device, accel.as_str(), (display_device == "virtio-vga-gl" && accel).as_str());

        Ok((vec!["-display".into(), display_render, "-device".into(), video], Some(vec![message])))
    }
}

fn display_resolution(name: Option<String>) -> Result<(u32, u32)> {
    let display_info = display_info::DisplayInfo::all()?;
    log::debug!("Displays: {:?}", display_info);
    let display = if let Some(monitor) = name {
        display_info.iter().find(|available| available.name == monitor).ok_or_else(|| anyhow!("Could not find a display matching the name {}", monitor))?
    } else {
        display_info.iter().find(|available| available.is_primary)
            .unwrap_or(display_info.first().ok_or_else(|| anyhow!("Could not find a monitor. Please manually specify the resolution in your config file."))?)
    };

    Ok((display.width, display.height))
}
