use std::{borrow::Cow, ffi::OsStr};

use crate::{
    data::{Arch, Display, DisplayType, GuestOS, Resolution},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs},
};

impl Display {
    pub(crate) fn args(&self, guest: GuestOS, arch: Arch) -> Result<(DisplayArgs, Vec<Warning>), Error> {
        let gpu = match arch {
            Arch::X86_64 { .. } => match guest {
                GuestOS::Linux => match self.display_type {
                    DisplayType::None => GpuType::VirtIOGPU,
                    #[cfg(not(target_os = "macos"))]
                    DisplayType::Spice { .. } | DisplayType::SpiceApp { .. } => GpuType::VirtIOGPU,
                    _ => GpuType::VirtIOVGA,
                },
                GuestOS::Windows | GuestOS::WindowsServer if self.display_type == DisplayType::Sdl => GpuType::VirtIOVGA,
                #[cfg(not(target_os = "macos"))]
                GuestOS::Windows | GuestOS::WindowsServer if matches!(self.display_type, DisplayType::SpiceApp { .. }) => GpuType::VirtIOVGA,
                #[cfg(target_os = "macos")]
                GuestOS::Windows | GuestOS::WindowsServer if self == &Self::Cocoa => GpuType::VirtIOVGA,
                GuestOS::Solaris | GuestOS::LinuxOld => GpuType::VMWareSVGA,
                _ => GpuType::Qxl,
            },
            Arch::AArch64 { .. } => GpuType::VirtIOGPU,
            Arch::Riscv64 { .. } => GpuType::VirtIOVGA,
        };

        let (fullscreen, res) = match &self.resolution {
            Resolution::FullScreen => (true, None),
            Resolution::Default => (false, display_resolution(None, None)),
            Resolution::Custom { width, height } => (false, Some((*width, *height))),
            Resolution::Display { display_name, percentage } => (false, display_resolution(display_name.as_deref(), *percentage)),
        };

        Ok((
            DisplayArgs {
                fullscreen,
                res,
                accelerated: self.accelerated.unwrap_or(default_accel()),
                gpu,
                display: self.display_type,
            },
            Vec::new(),
        ))
    }
}

fn display_resolution(name: Option<&str>, screenpct: Option<f64>) -> Option<(u32, u32)> {
    let display_info = display_info::DisplayInfo::all().ok()?;
    log::debug!("Displays: {:?}", display_info);
    let display = if let Some(monitor) = name {
        display_info.iter().find(|available| available.name == monitor)
    } else {
        display_info
            .iter()
            .find(|available| available.is_primary)
            .or(display_info.first())
    }?;

    let (width, height) = match (display.width, display.height, screenpct) {
        (width, height, Some(screenpct)) => (
            (screenpct * width as f64 / 100.0) as u32,
            (screenpct * height as f64 / 100.0) as u32,
        ),
        (3840.., 2160.., _) => (3200, 1800),
        (2560.., 1440.., _) => (2048, 1152),
        (1920.., 1080.., _) => (1664, 936),
        (1280.., 800.., _) => (1152, 648),
        (width, height, _) => (width, height),
    };

    Some((width, height))
}

pub(crate) struct DisplayArgs {
    res: Option<(u32, u32)>,
    fullscreen: bool,
    accelerated: bool,
    gpu: GpuType,
    display: DisplayType,
}

#[derive(PartialEq, derive_more::Display)]
enum GpuType {
    VirtIOVGA,
    VirtIOGPU,
    VMWareSVGA,
    Qxl,
}

impl EmulatorArgs for DisplayArgs {
    fn display(&self) -> Option<ArgDisplay> {
        let resolution_text = if self.gpu != GpuType::VMWareSVGA && !self.fullscreen && self.res.is_some() {
            let (x, y) = self.res.unwrap();
            format!(", Resolution: {x}x{y}")
        } else {
            "".into()
        };
        Some(ArgDisplay {
            name: "Display".into(),
            value: format!(
                "{}, Device: {}, GL: {}{}",
                self.display, self.gpu, self.accelerated, resolution_text
            )
            .into(),
        })
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = crate::utils::QemuArg> {
        let mut args = Vec::new();
        let display_device_arg = match self.gpu {
            GpuType::VirtIOGPU => "virtio-gpu",
            GpuType::VirtIOVGA if self.accelerated => "virtio-vga-gl",
            GpuType::VirtIOVGA => "virtio-vga",
            GpuType::VMWareSVGA => "vmware-svga,vgamem_mb=256",
            GpuType::Qxl => "qxl-vga,ram_size=65536,vram_size=65536,vgamem_mb=64",
        };

        args.push(Cow::Borrowed(OsStr::new("-display")));
        let gl_text = if self.accelerated { "on" } else { "off" };
        args.push(match self.display {
            DisplayType::Gtk => Cow::Owned(format!("gtk,grab-on-hover=on,zoom-to-fit=off,gl={gl_text}").into()),
            DisplayType::None => Cow::Borrowed(OsStr::new("none")),
            DisplayType::Sdl => Cow::Owned(format!("sdl,gl={gl_text}").into()),
            #[cfg(not(target_os = "macos"))]
            DisplayType::Spice { .. } => Cow::Borrowed(OsStr::new("none")),
            #[cfg(not(target_os = "macos"))]
            DisplayType::SpiceApp { .. } => Cow::Owned(format!("spice-app,gl={gl_text}").into()),
            #[cfg(target_os = "macos")]
            DisplayType::Cocoa => Cow::Borrowed(OsStr::new("cocoa")),
        });
        args.push(Cow::Borrowed(OsStr::new("-vga")));
        args.push(Cow::Borrowed(OsStr::new("none")));

        args.push(Cow::Borrowed(OsStr::new("-display")));
        if self.fullscreen || self.gpu == GpuType::VMWareSVGA || self.res.is_none() {
            args.push(Cow::Borrowed(OsStr::new(display_device_arg)));
        } else {
            let (x, y) = self.res.unwrap();
            args.push(Cow::Owned(format!("{display_device_arg},xres={x},yres={y}").into()));
        }
        if self.fullscreen {
            args.push(Cow::Borrowed(OsStr::new("-full-screen")));
        }
        args
    }
}

fn default_accel() -> bool {
    cfg!(not(target_os = "macos"))
}
