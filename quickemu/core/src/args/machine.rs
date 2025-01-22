use std::path::Path;

use cpu::Cpu;
use itertools::chain;
use ram::Ram;
use tpm::Tpm;

use crate::{
    data::{GuestOS, Machine},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, LaunchFn, QemuArg},
};

mod cpu;
mod ram;
mod tpm;

impl Machine {
    pub fn args(&self, guest: GuestOS, vm_dir: &Path, vm_name: &str) -> Result<(MachineArgs, Vec<Warning>), Error> {
        let mut warnings = Vec::new();
        let (cpu_args, cpu_warnings) = self.cpu_args(guest)?;
        warnings.extend(cpu_warnings);

        let (ram_args, ram_warning) = self.ram_args(guest)?;
        warnings.extend(ram_warning);

        let tpm_args = self.tpm.then(|| Tpm::new(vm_dir, vm_name)).transpose()?;

        Ok((MachineArgs { cpu_args, ram_args, tpm_args }, warnings))
    }
}

pub struct MachineArgs {
    cpu_args: Cpu,
    ram_args: Ram,
    tpm_args: Option<Tpm>,
}

impl EmulatorArgs for MachineArgs {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        chain!(
            self.cpu_args.display(),
            self.ram_args.display(),
            self.tpm_args.as_ref().map(|tpm| tpm.display()).into_iter().flatten()
        )
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        chain!(
            self.cpu_args.qemu_args(),
            self.ram_args.qemu_args(),
            self.tpm_args.as_ref().map(|tpm| tpm.qemu_args()).into_iter().flatten()
        )
    }
    fn launch_fns(self) -> impl IntoIterator<Item = LaunchFn> {
        chain!(
            self.cpu_args.launch_fns(),
            self.ram_args.launch_fns(),
            self.tpm_args.map(|tpm| tpm.launch_fns()).into_iter().flatten(),
        )
    }
}
