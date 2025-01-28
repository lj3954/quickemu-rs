use std::{collections::HashSet, path::Path};

use disks::DiskArgs;

use crate::{
    data::{GuestOS, Images},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

mod disks;

pub(crate) struct ImageArgs<'a> {
    disks: DiskArgs<'a>,
}

impl Images {
    pub(crate) fn args(&self, guest: GuestOS, vm_dir: &Path, status_quo: bool) -> Result<(ImageArgs, Option<Warning>), Error> {
        let mut used_indices = HashSet::new();
        let disks = self.disk_args(guest, vm_dir, status_quo, &mut used_indices)?;

        Ok((ImageArgs { disks }, None))
    }
}

impl EmulatorArgs for ImageArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        self.disks.display()
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        self.disks.qemu_args()
    }
}
