use std::{borrow::Cow, process::Command};

use which::which;

use crate::{
    arg,
    data::{Accelerated, Access, Display, DisplayType, GuestOS, Resolution, Viewer},
    error::Error,
    oarg,
    utils::{find_port, ArgDisplay, EmulatorArgs, LaunchFn, LaunchFnReturn, QemuArg},
};

impl<'a> Display {
    pub fn spice_args(&self, vm_name: &'a str, guest: GuestOS, public_dir: Option<Cow<'a, str>>) -> Result<SpiceArgs<'a>, Error> {
        match self.display_type {
            DisplayType::SpiceApp => Ok(SpiceArgs::SpiceApp { accelerated: self.accelerated }),
            DisplayType::Spice { access, spice_port, viewer } => {
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
                    viewer,
                })
            }
            _ => unreachable!(),
        }
    }
}

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
        viewer: Viewer,
    },
}

impl EmulatorArgs for SpiceArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        Some(match self {
            Self::SpiceApp { .. } => ArgDisplay {
                name: Cow::Borrowed("Spice"),
                value: Cow::Borrowed("Enabled"),
            },
            Self::Spice {
                vm_name,
                port,
                public_dir,
                viewer,
                fullscreen,
                ..
            } => {
                let mut msg = match viewer {
                    Viewer::None => return None,
                    Viewer::Spicy => format!("spicy --title \"{vm_name}\" --port {port}"),
                    Viewer::Remote => format!("remote-viewer --title \"{vm_name}\" \"spice://localhost:{port}\""),
                };
                if let Some(public_dir) = public_dir {
                    msg.extend([" --spice-shared-dir ", public_dir]);
                }
                if *fullscreen {
                    msg.push_str(" --full-screen");
                }
                ArgDisplay {
                    name: Cow::Borrowed("Viewer (On host)"),
                    value: Cow::Owned(msg),
                }
            }
        })
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
        [arg!("-spice"), oarg!(spice_arg)]
    }

    fn launch_fns(self) -> impl IntoIterator<Item = LaunchFn> {
        match self {
            Self::Spice {
                viewer,
                vm_name,
                public_dir,
                fullscreen,
                port,
                ..
            } => {
                if let Viewer::None = viewer {
                    return None;
                }
                let vm_name = vm_name.to_string();
                let public_dir = public_dir.map(|d| d.to_string());

                let launch = move || {
                    let viewer_cmd = match viewer {
                        Viewer::Spicy => "spicy",
                        Viewer::Remote => "remote-viewer",
                        _ => unreachable!(),
                    };
                    let cmd = which(viewer_cmd).map_err(|_| Error::ViewerNotFound(viewer_cmd))?;

                    #[cfg(not(feature = "inbuilt_commands"))]
                    let mut command = Command::new(cmd);

                    command.arg("--title").arg(vm_name);
                    match viewer {
                        Viewer::Spicy => command.arg("--port").arg(port.to_string()),
                        Viewer::Remote => command.arg(format!("spice://localhost:{port}")),
                        _ => unreachable!(),
                    };

                    if let Some(public_dir) = public_dir {
                        command.arg("--spice-shared-dir");
                        command.arg(public_dir);
                    }

                    if fullscreen {
                        command.arg("--full-screen");
                    }

                    let child = command.spawn().map_err(|e| Error::Command(viewer_cmd, e.to_string()))?;

                    Ok(vec![LaunchFnReturn::Process(child)])
                };

                Some(LaunchFn::After(Box::new(launch)))
            }
            _ => None,
        }
    }
}
