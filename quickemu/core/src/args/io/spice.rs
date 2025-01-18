use std::{borrow::Cow, ffi::OsStr};

use crate::{
    data::{Accelerated, Access, Display, DisplayType, GuestOS, Resolution},
    error::Error,
    utils::{find_port, ArgDisplay, EmulatorArgs, QemuArg},
};

#[cfg(not(target_os = "macos"))]
impl<'a> Display {
    pub fn spice_args(&self, vm_name: &'a str, guest: GuestOS, public_dir: Option<Cow<'a, str>>) -> Result<SpiceArgs<'a>, Error> {
        match self.display_type {
            DisplayType::SpiceApp => Ok(SpiceArgs::SpiceApp { accelerated: self.accelerated }),
            DisplayType::Spice { access, spice_port, .. } => {
                let Some(port) = find_port(spice_port, 9) else {
                    return Err(Error::UnavailablePort(spice_port));
                };
                let public_dir = public_dir.and_then(|dir| (!matches!(guest, GuestOS::MacOS { .. })).then_some(dir));
                let fullscreen = matches!(self.resolution, Resolution::FullScreen);
                Ok(SpiceArgs::Spice {
                    fullscreen,
                    vm_name,
                    port,
                    access,
                    public_dir,
                })
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub enum SpiceArgs<'a> {
    SpiceApp {
        accelerated: Accelerated,
    },
    Spice {
        fullscreen: bool,
        vm_name: &'a str,
        port: u16,
        access: Access,
        public_dir: Option<Cow<'a, str>>,
    },
}

#[cfg(not(target_os = "macos"))]
impl EmulatorArgs for SpiceArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let name = Cow::Borrowed("Spice");
        let value = match self {
            Self::SpiceApp { .. } => Cow::Borrowed("Enabled"),
            Self::Spice { vm_name, port, public_dir, .. } => {
                let mut msg = format!("On host: spicy --title \"{vm_name}\" --port {port}");
                if let Some(public_dir) = public_dir {
                    msg.extend([" --spice-shared-dir ", public_dir]);
                }
                Cow::Owned(msg)
            }
        };
        Some(ArgDisplay { name, value })
    }

    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut spice_arg = "disable-ticketing=on".to_string();
        match self {
            Self::SpiceApp { accelerated } => {
                spice_arg.push_str(&format!(",gl={}", accelerated.as_ref()));
            }
            Self::Spice { port, access, .. } => {
                spice_arg.extend([",port=", &port.to_string()]);
                if let Some(address) = access.as_ref() {
                    spice_arg.extend([",addr=", &address.to_string()]);
                }
            }
        }
        [Cow::Borrowed(OsStr::new("-spice")), Cow::Owned(spice_arg.into())]
    }
}
