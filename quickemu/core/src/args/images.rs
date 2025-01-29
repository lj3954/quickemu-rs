use std::{collections::HashSet, path::Path};

use disks::DiskArgs;
use iso::IsoArgs;
use itertools::chain;

use crate::{
    data::{GuestOS, Images},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, LaunchFn, QemuArg},
};

mod disks;
mod iso;

impl Images {
    pub(crate) fn args(&self, guest: GuestOS, vm_dir: &Path, status_quo: bool) -> Result<(ImageArgs, Option<Warning>), Error> {
        let mut used_indices = HashSet::new();
        let disks = self.disk_args(guest, vm_dir, status_quo, &mut used_indices)?;
        let isos = self.iso_args(disks.installed(), guest, vm_dir, &mut used_indices)?;

        Ok((ImageArgs { disks, isos }, None))
    }
}

pub(crate) struct ImageArgs<'a> {
    disks: DiskArgs<'a>,
    isos: IsoArgs<'a>,
}

impl EmulatorArgs for ImageArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        chain!(self.disks.display(), self.isos.display())
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        chain!(self.disks.qemu_args(), self.isos.qemu_args())
    }
    fn launch_fns(self) -> impl IntoIterator<Item = LaunchFn> {
        chain!(self.disks.launch_fns(), self.isos.launch_fns())
    }
}
