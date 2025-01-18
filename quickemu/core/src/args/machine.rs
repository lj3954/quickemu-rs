use cpu::Cpu;
use itertools::chain;
use ram::Ram;

use crate::{
    data::{GuestOS, Machine},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

mod cpu;
mod ram;

impl Machine {
    pub fn args(&self, guest: GuestOS) -> Result<(MachineArgs, Vec<Warning>), Error> {
        let mut warnings = Vec::new();
        let (cpu_args, cpu_warnings) = self.cpu_args(guest)?;
        warnings.extend(cpu_warnings);

        let (ram_args, ram_warning) = self.ram_args(guest)?;
        warnings.extend(ram_warning);

        Ok((MachineArgs { cpu_args, ram_args }, warnings))
    }
}

pub struct MachineArgs {
    cpu_args: Cpu,
    ram_args: Ram,
}

impl EmulatorArgs for MachineArgs {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        chain!(self.cpu_args.display(), self.ram_args.display())
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        chain!(self.cpu_args.qemu_args(), self.ram_args.qemu_args())
    }
}
