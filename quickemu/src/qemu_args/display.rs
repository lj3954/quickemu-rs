use anyhow::{anyhow, bail, Result};
use std::ffi::OsString;
use crate::config::{Access, Arch, BooleanDisplay, Display, Monitor, SoundCard, GuestOS, Resolution, Viewer};
use std::net::TcpStream;
use std::os::unix::net::UnixStream;
use std::io::{Write, Read};
use std::process::Command;
use crate::qemu_args::find_port;
use which::which;

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
            #[cfg(not(target_os = "macos"))]
            Self::None | Self::Spice | Self::SpiceApp => ["-audiodev".into(), "spice,id=audio0".into()],
            #[cfg(target_os = "macos")]
            _ => ["-audiodev".into(), "coreaudio,id=audio0".into()],
            #[cfg(not(target_os = "macos"))]
            _ => ["-audiodev".into(), "pa,id=audio0".into()],
        }
    }

    pub fn display_args(&self, guest_os: &GuestOS, arch: &Arch, resolution: Resolution, screenpct: Option<u32>, accel: bool, fullscreen: bool) -> Result<(Vec<String>, Option<Vec<String>>)> {
        let virtio_vga = || if accel { "virtio-vga-gl" } else { "virtio-vga" };
        let (display_device, friendly_display_device) = match arch {
            Arch::x86_64 => match guest_os {
                GuestOS::Linux => match self {
                    Self::None => ("virtio-gpu", "VirtIO GPU"),
                    #[cfg(not(target_os = "macos"))]
                    Self::Spice | Self::SpiceApp => ("virtio-gpu", "VirtIO GPU"),
                    _ => (virtio_vga(), "VirtIO VGA"),
                },
                GuestOS::Windows | GuestOS::WindowsServer if self == &Self::Sdl => (virtio_vga(), "VirtIO VGA"),
                #[cfg(not(target_os = "macos"))]
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
            #[cfg(not(target_os = "macos"))]
            Self::Spice => "none".to_string(),
            #[cfg(not(target_os = "macos"))]
            Self::SpiceApp => "spice-app,gl=".to_string() + gl,
            #[cfg(target_os = "macos")]
            Self::Cocoa => "cocoa".to_string(),
        };

        let mut message = format!("Display: {}, Device: {}, GL: {}, VirGL: {}", self, friendly_display_device, accel.as_str(), (display_device == "virtio-vga-gl").as_str());

        let video = if matches!(guest_os, GuestOS::LinuxOld | GuestOS::Solaris) || fullscreen {
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
        let mut args = vec!["-display".into(), display_render, "-device".into(), video, "-vga".into(), "none".into()];
        if fullscreen {
            args.push("--full-screen".into());
        }
        Ok((args, Some(vec![message])))
    }

    #[cfg(not(target_os = "macos"))]
    pub fn spice_args(&self, port: Option<u16>, access: Access, custom_params: (bool, bool), guest_os: &GuestOS, publicdir: Option<&OsString>, vm_name: &str) -> Result<(Vec<String>, Option<Vec<String>>)> {
        let mut spice = "disable-ticketing=on".to_string();
        match self {
            Self::SpiceApp => {
                let gl = if custom_params.0 { "on" } else { "off" };
                spice.extend([",gl=", gl]);
                Ok((vec!["-spice".into(), spice], Some(vec!["Spice: Enabled".into()])))
            },
            _ => {
                let spice_addr = match access {
                    Access::Remote => "".into(),
                    Access::Local => "127.0.0.1".into(),
                    Access::Address(address) => address,
                };
                let port = port.ok_or_else(|| anyhow!("Requested SPICE display, but no ports are available."))?.to_string();
                spice.extend([",port=", &port, ",addr=", &spice_addr]);

                let mut msg = "Spice: On host: spicy --title \"".to_string() + vm_name + "\" --port " + &port;
                match (publicdir, guest_os) {
                    (None, _) | (_, GuestOS::MacOS { .. }) => (),
                    (Some(dir), _) => msg.extend([" --spice-shared-dir ", &dir.to_string_lossy()]),
                }
                if custom_params.1 {
                    msg.push_str(" --full-screen");
                }
                Ok((vec!["-spice".into(), spice], Some(vec![msg])))
            },
        }
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
            Self::Telnet { address } => {
                let port = find_port(address.port(), 9).ok_or_else(|| anyhow!("Could not find an open port for the telnet monitor."))?;
                let address = address.ip().to_string() + ":" + &port.to_string();
                let mut telnet = OsString::from("telnet:");
                telnet.push(&address);
                telnet.push(",server,nowait");
                (vec![arg, telnet], Some(vec![text + "On host: telnet " + &address]))
            },
            Self::Socket { socketpath } => {
                let mut socket = OsString::from("unix:");
                socket.push(socketpath);
                socket.push(",server,nowait");
                (vec![arg, socket], Some(vec![text + "On host: " + &socketpath.to_string_lossy()]))
            },
        })
    }

    pub fn send_command(&self, command: &str) -> Result<()> {
        let command = command.to_string() + "\n";
        match self {
            Self::None => bail!("A command was requested to be sent to the guest, but no monitor is enabled."),
            Self::Telnet { address } => {
                let mut stream = TcpStream::connect(address)?;
                stream.write_all(command.as_bytes())?;
                stream.shutdown(std::net::Shutdown::Write)?;
                stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;
                let mut response = String::new();
                stream.read_to_string(&mut response)?;
                log::debug!("Received response: {}", response);
                log::debug!("Sent command {} to address {:?}", command, stream);
            },
            Self::Socket { socketpath } => {
                let mut stream = UnixStream::connect(socketpath)?;
                stream.write_all(command.as_bytes())?;
                stream.shutdown(std::net::Shutdown::Write)?;
                stream.set_read_timeout(Some(std::time::Duration::from_secs(1)))?;
                let mut response = String::new();
                stream.read_to_string(&mut response)?;
                log::debug!("Received response: {}", response);
                log::debug!("Sent command {} to socket {:?}", command, stream);
            },
        };
        Ok(())
    }
}

impl Viewer {
    #[cfg(not(target_os = "macos"))]
    pub fn start(&self, vm_name: &str, publicdir: Option<&OsString>, fullscreen: bool, port: u16) -> Result<()> {
        match self {
            Self::None => (),
            Self::Remote => {
                let viewer = which("remote-viewer").map_err(|_| anyhow!("Remote viewer was selected, but remote-viewer is not installed."))?;
                let publicdir = publicdir.map(|dir| dir.to_string_lossy()).unwrap_or_default();
                println!(r#" - Viewer: remote-viewer --title "{}" --spice-shared-dir "{}"{} "spice://localhost:{}""#, vm_name, publicdir, if fullscreen { " --full-screen" } else { "" }, port);
                
                Command::new(viewer).arg("--title").arg(vm_name).arg("--spice-shared-dir").arg(publicdir.to_string()).arg(if fullscreen { "--full-screen" } else { "" }).arg(format!("spice://localhost:{}", port)).spawn()
                    .map_err(|e| anyhow!("Could not start viewer: {}", e))?;
            },
            Self::Spicy => {
                let viewer = which("spicy").map_err(|_| anyhow!("Spicy is not installed, spicy viewer cannot be used."))?;
                let publicdir = publicdir.map(|dir| dir.to_string_lossy()).unwrap_or_default();
                println!(r#" - Viewer: spicy --title "{}" --port {} --spice-shared-dir "{}"{}"#, vm_name, port, publicdir, if fullscreen { " --full-screen" } else { "" });

                Command::new(viewer).arg("--title").arg(vm_name).arg("--port").arg(port.to_string()).arg("--spice-shared-dir").arg(publicdir.to_string()).arg(if fullscreen { "--full-screen" } else { "" }).spawn()
                    .map_err(|e| anyhow!("Could not start viewer: {}", e))?;
            },
        }
        Ok(())
    }
}
