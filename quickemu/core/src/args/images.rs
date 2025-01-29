use std::{borrow::Cow, collections::HashSet, path::Path, thread, time::Duration};

use disks::DiskArgs;
use img::ImgArgs;
use iso::IsoArgs;
use itertools::chain;

use crate::{
    data::{GuestOS, Images, Monitor},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, LaunchFn, LaunchFnReturn, QemuArg},
};

mod disks;
mod img;
mod iso;

impl Images {
    pub(crate) fn args(&self, guest: GuestOS, vm_dir: &Path, status_quo: bool, monitor: Monitor) -> Result<(ImageArgs, Option<Warning>), Error> {
        let mut used_indices = HashSet::new();
        let disks = self.disk_args(guest, vm_dir, status_quo, &mut used_indices)?;
        let isos = self.iso_args(disks.installed(), guest, vm_dir, &mut used_indices)?;
        let imgs = self.img_args(disks.installed(), vm_dir, guest)?;

        let monitor_cmds = matches!(guest, GuestOS::Windows).then(|| MonitorCmds {
            monitor,
            cmds: (0..5)
                .map(|_| MonitorCmd {
                    wait_before: Duration::from_secs(1),
                    command: Cow::Borrowed("sendkey ret"),
                })
                .collect(),
        });

        Ok((ImageArgs { disks, isos, imgs, monitor_cmds }, None))
    }
}

pub(crate) struct ImageArgs<'a> {
    disks: DiskArgs<'a>,
    isos: IsoArgs<'a>,
    imgs: ImgArgs<'a>,
    monitor_cmds: Option<MonitorCmds>,
}

impl EmulatorArgs for ImageArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        chain!(
            self.disks.display(),
            self.isos.display(),
            self.imgs.display(),
            self.monitor_cmds.as_ref().map(|cmds| cmds.display()).into_iter().flatten(),
        )
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        chain!(
            self.disks.qemu_args(),
            self.isos.qemu_args(),
            self.imgs.qemu_args(),
            self.monitor_cmds.as_ref().map(|cmds| cmds.qemu_args()).into_iter().flatten(),
        )
    }
    fn launch_fns(self) -> impl IntoIterator<Item = LaunchFn> {
        chain!(
            self.disks.launch_fns(),
            self.isos.launch_fns(),
            self.imgs.launch_fns(),
            self.monitor_cmds.map(|cmds| cmds.launch_fns()).into_iter().flatten()
        )
    }
}

struct MonitorCmds {
    monitor: Monitor,
    cmds: Vec<MonitorCmd>,
}

struct MonitorCmd {
    wait_before: Duration,
    command: Cow<'static, str>,
}

impl EmulatorArgs for MonitorCmds {
    fn launch_fns(self) -> impl IntoIterator<Item = LaunchFn> {
        let launch_fn = Box::new(move || {
            let thread = thread::spawn(move || {
                for cmd in self.cmds {
                    thread::sleep(cmd.wait_before);
                    log::info!("Sending monitor command: {}", cmd.command);
                    self.monitor
                        .send_cmd(&cmd.command)
                        .map_err(|e| Error::MonitorCommand(e.to_string()))?;
                }
                Ok(())
            });

            Ok(vec![LaunchFnReturn::Thread(thread)])
        });

        Some(LaunchFn::After(launch_fn))
    }
}
