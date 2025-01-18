use std::{borrow::Cow, ffi::OsStr};

use size::Size;

use crate::{
    data::{GuestOS, Machine},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

pub struct Ram {
    ram: Size,
    total_ram: Size,
    free_ram: Size,
}

const MIN_MACOS_WINDOWS_RAM: i64 = 4 * size::consts::GiB;

impl Machine {
    pub fn ram_args(&self, guest: GuestOS) -> Result<(Ram, Option<Warning>), Error> {
        let mut warning = None;

        let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_memory(sysinfo::MemoryRefreshKind::new().with_ram()));
        let free_ram = Size::from_bytes(system.available_memory());
        let total_ram = Size::from_bytes(system.total_memory());

        let mut ram = self.ram.map_or_else(|| match total_ram.bytes() / size::consts::GiB {
            128.. => 32,
            64.. => 16,
            16.. => 8,
            8.. => 4,
            4.. => 2,
            _ => 1,
        } * size::consts::GiB, |ram| ram as i64);

        if ram < MIN_MACOS_WINDOWS_RAM {
            if self.ram.is_some() {
                warning = Some(Warning::InsufficientRamConfiguration(total_ram, guest));
            } else if total_ram.bytes() < MIN_MACOS_WINDOWS_RAM {
                return Err(Error::InsufficientRam(total_ram, guest));
            } else {
                ram = MIN_MACOS_WINDOWS_RAM;
            }
        }

        let ram = Size::from_bytes(ram);

        Ok((Ram { ram, total_ram, free_ram }, warning))
    }
}

impl EmulatorArgs for Ram {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        Some(ArgDisplay {
            name: Cow::Borrowed("RAM"),
            value: Cow::Owned(format!("{} ({} / {} available)", self.ram, self.free_ram, self.total_ram)),
        })
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        [Cow::Borrowed(OsStr::new("-m")), Cow::Owned(format!("{}b", self.ram.bytes()).into())]
    }
}
