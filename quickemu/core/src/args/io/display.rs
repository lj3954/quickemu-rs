use crate::{
    arg,
    data::{Accelerated, Arch, Display, DisplayType, GuestOS, Resolution},
    error::Error,
    oarg,
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

impl Display {
    pub(crate) fn args(&self, guest: GuestOS, arch: Arch) -> Result<DisplayArgs, Error> {
        let gpu = match arch {
            Arch::X86_64 { .. } => match guest {
                GuestOS::Linux => match self.display_type {
                    DisplayType::None => GpuType::VirtIOGPU,
                    #[cfg(not(target_os = "macos"))]
                    DisplayType::Spice { .. } | DisplayType::SpiceApp => GpuType::VirtIOGPU,
                    _ => GpuType::VirtIOVGA,
                },
                GuestOS::Windows | GuestOS::WindowsServer if self.display_type == DisplayType::Sdl => GpuType::VirtIOVGA,
                #[cfg(not(target_os = "macos"))]
                GuestOS::Windows | GuestOS::WindowsServer if matches!(self.display_type, DisplayType::SpiceApp) => GpuType::VirtIOVGA,
                #[cfg(target_os = "macos")]
                GuestOS::Windows | GuestOS::WindowsServer if self.display_type == DisplayType::Cocoa => GpuType::VirtIOVGA,
                GuestOS::Solaris | GuestOS::LinuxOld => GpuType::VMwareSVGA,
                _ => GpuType::Qxl,
            },
            Arch::AArch64 { .. } => GpuType::VirtIOGPU,
            Arch::Riscv64 { .. } => GpuType::VirtIOVGA,
        };

        let (fullscreen, res) = match &self.resolution {
            Resolution::FullScreen => (true, None),
            #[cfg(feature = "display_resolution")]
            Resolution::Default => (false, display_resolution(None, None)),
            #[cfg(not(feature = "display_resolution"))]
            Resolution::Default => (false, Some((1280, 800))),
            Resolution::Custom { width, height } => (false, Some((*width, *height))),
            #[cfg(feature = "display_resolution")]
            Resolution::Display { display_name, percentage } => (false, display_resolution(display_name.as_deref(), *percentage)),
        };

        Ok(DisplayArgs {
            fullscreen,
            res,
            accelerated: self.accelerated,
            gpu,
            display: self.display_type,
            braille: self.braille,
        })
    }
}

#[cfg(feature = "display_resolution")]
fn display_resolution(name: Option<&str>, screenpct: Option<f64>) -> Option<(u32, u32)> {
    let display_info = display_info::DisplayInfo::all().ok()?;
    log::debug!("Displays: {display_info:?}");
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
    accelerated: Accelerated,
    gpu: GpuType,
    display: DisplayType,
    braille: bool,
}

#[derive(PartialEq, derive_more::Display)]
enum GpuType {
    #[display("VirtIO VGA")]
    VirtIOVGA,
    #[display("VirtIO GPU")]
    VirtIOGPU,
    #[display("VMware SVGA")]
    VMwareSVGA,
    #[display("QXL")]
    Qxl,
}

impl EmulatorArgs for DisplayArgs {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let resolution_text = if self.gpu != GpuType::VMwareSVGA && !self.fullscreen && self.res.is_some() {
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
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut args = Vec::new();
        let display_device_arg = match self.gpu {
            GpuType::VirtIOGPU => "virtio-gpu",
            GpuType::VirtIOVGA if self.accelerated.into() => "virtio-vga-gl",
            GpuType::VirtIOVGA => "virtio-vga",
            GpuType::VMwareSVGA => "vmware-svga,vgamem_mb=256",
            GpuType::Qxl => "qxl-vga,ram_size=65536,vram_size=65536,vgamem_mb=64",
        };

        let display_type_arg = match self.display {
            DisplayType::Gtk => oarg!(format!("gtk,grab-on-hover=on,zoom-to-fit=off,gl={}", self.accelerated.as_ref())),
            DisplayType::None => arg!("none"),
            DisplayType::Sdl => oarg!(format!("sdl,gl={}", self.accelerated.as_ref())),
            #[cfg(not(target_os = "macos"))]
            DisplayType::Spice { .. } => arg!("none"),
            #[cfg(not(target_os = "macos"))]
            DisplayType::SpiceApp => oarg!(format!("spice-app,gl={}", self.accelerated.as_ref())),
            #[cfg(target_os = "macos")]
            DisplayType::Cocoa => arg!("cocoa"),
        };
        args.extend([arg!("-display"), display_type_arg]);
        args.extend([arg!("-vga"), arg!("none")]);

        let display_device_arg = if self.fullscreen || self.gpu == GpuType::VMwareSVGA || self.res.is_none() {
            arg!(display_device_arg)
        } else {
            let (x, y) = self.res.unwrap();
            oarg!(format!("{display_device_arg},xres={x},yres={y}"))
        };
        args.extend([arg!("-device"), display_device_arg]);

        if self.fullscreen {
            args.push(arg!("-full-screen"));
        }

        if self.braille {
            args.extend([arg!("-chardev"), arg!("braille,id=brltty"), arg!("-device"), arg!("usb-braille,id=usbbrl,chardev=brltty")]);
        }
        args
    }
}
