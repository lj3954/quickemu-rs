use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
};

use which::which;

use crate::{
    error::Error,
    utils::{ArgDisplay, EmulatorArgs, LaunchFn, LaunchFnReturn},
};

#[cfg(not(feature = "inbuilt_commands"))]
use std::process::Command;

impl Tpm {
    pub(crate) fn new(vm_dir: &Path, vm_name: &str) -> Result<Tpm, Error> {
        #[cfg(not(feature = "inbuilt_commands"))]
        let binary = which("swtpm")?;

        let socket = vm_dir.join(format!("{}.swtpm-sock", vm_name));

        let mut ctrl = OsString::from("type=unixio,path=");
        ctrl.push(&socket);

        let mut tpmstate = OsString::from("dir=");
        tpmstate.push(vm_dir);

        Ok(Tpm {
            #[cfg(not(feature = "inbuilt_commands"))]
            binary,
            ctrl,
            tpmstate,
            socket,
        })
    }
}

pub(crate) struct Tpm {
    #[cfg(not(feature = "inbuilt_commands"))]
    binary: PathBuf,
    ctrl: OsString,
    tpmstate: OsString,
    socket: PathBuf,
}

impl EmulatorArgs for Tpm {
    fn launch_fn(self) -> Option<LaunchFn> {
        Some(Box::new(move || {
            let tpm_args = [
                OsStr::new("socket"),
                OsStr::new("--ctrl"),
                &self.ctrl,
                OsStr::new("--terminate"),
                OsStr::new("--tpmstate"),
                &self.tpmstate,
                OsStr::new("--tpm2"),
            ];

            let args = |socket: PathBuf| {
                [
                    Cow::Borrowed(OsStr::new("-chardev")),
                    Cow::Owned(socket.into_os_string()),
                    Cow::Borrowed(OsStr::new("-tpmdev")),
                    Cow::Borrowed(OsStr::new("emulator,id=tpm0,chardev=chrtpm")),
                    Cow::Borrowed(OsStr::new("-device")),
                    Cow::Borrowed(OsStr::new("tpm-tis,tpmdev=tpm0")),
                ]
                .into_iter()
                .map(LaunchFnReturn::Arg)
            };

            #[cfg(not(feature = "inbuilt_commands"))]
            {
                let child = Command::new(&self.binary)
                    .args(tpm_args)
                    .spawn()
                    .map_err(|e| Error::Command("swtpm", e.to_string()))?;
                let pid = child.id();

                Ok([
                    LaunchFnReturn::Process(child),
                    LaunchFnReturn::Display(ArgDisplay {
                        name: Cow::Borrowed("TPM"),
                        value: Cow::Owned(format!("{} (pid: {})", self.socket.display(), pid)),
                    }),
                ]
                .into_iter()
                .chain(args(self.socket))
                .collect())
            }
            #[cfg(feature = "inbuilt_commands")]
            {
                todo!()
            }
        }))
    }
}