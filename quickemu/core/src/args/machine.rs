use std::{ffi::OsString, path::Path};

use boot::BootArgs;
use cpu::Cpu;
use itertools::chain;
use ram::Ram;
use tpm::Tpm;

use crate::{
    arg,
    data::{AArch64Machine, Arch, GuestOS, Machine, Riscv64Machine, X86_64Machine},
    error::{Error, Warning},
    oarg,
    utils::{ArgDisplay, EmulatorArgs, LaunchFn, QemuArg},
};

mod boot;
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
        let boot_args = self.boot_args(vm_dir, guest)?;
        let machine_type = FullMachine::new(self.arch, guest);

        Ok((
            MachineArgs {
                cpu_args,
                ram_args,
                tpm_args,
                boot_args,
                machine_type,
            },
            warnings,
        ))
    }
}

pub struct MachineArgs {
    cpu_args: Cpu,
    ram_args: Ram,
    tpm_args: Option<Tpm>,
    boot_args: BootArgs,
    machine_type: FullMachine,
}

impl EmulatorArgs for MachineArgs {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        chain!(
            self.cpu_args.display(),
            self.ram_args.display(),
            self.tpm_args.as_ref().map(|tpm| tpm.display()).into_iter().flatten(),
            self.boot_args.display(),
            self.machine_type.display(),
        )
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        chain!(
            self.cpu_args.qemu_args(),
            self.ram_args.qemu_args(),
            self.tpm_args.as_ref().map(|tpm| tpm.qemu_args()).into_iter().flatten(),
            self.boot_args.qemu_args(),
            self.machine_type.qemu_args(),
        )
    }
    fn launch_fns(self) -> impl IntoIterator<Item = LaunchFn> {
        chain!(
            self.cpu_args.launch_fns(),
            self.ram_args.launch_fns(),
            self.tpm_args.map(|tpm| tpm.launch_fns()).into_iter().flatten(),
            self.boot_args.launch_fns(),
            self.machine_type.launch_fns(),
        )
    }
}

struct FullMachine {
    qemu_machine: QemuMachineType,
    specific: MachineType,
}

enum MachineType {
    X86_64 { smm: bool, no_hpet: bool },
    AArch64,
    Riscv64,
}

impl FullMachine {
    fn new(arch: Arch, guest: GuestOS) -> Self {
        match arch {
            Arch::X86_64 { machine: X86_64Machine::Standard } => {
                let smm = matches!(guest, GuestOS::Windows | GuestOS::WindowsServer | GuestOS::FreeDOS);
                let no_hpet = matches!(guest, GuestOS::Windows | GuestOS::WindowsServer | GuestOS::MacOS { .. });
                let qemu_machine_type = match guest {
                    GuestOS::FreeDOS | GuestOS::Batocera | GuestOS::Haiku | GuestOS::Solaris | GuestOS::ReactOS | GuestOS::KolibriOS => QemuMachineType::Pc,
                    _ => QemuMachineType::Qemu32,
                };
                Self {
                    qemu_machine: qemu_machine_type,
                    specific: MachineType::X86_64 { smm, no_hpet },
                }
            }
            Arch::AArch64 { machine: AArch64Machine::Standard } => Self {
                qemu_machine: QemuMachineType::Virt,
                specific: MachineType::AArch64,
            },
            Arch::Riscv64 { machine: Riscv64Machine::Standard } => Self {
                qemu_machine: QemuMachineType::Virt,
                specific: MachineType::Riscv64,
            },
        }
    }
}

impl EmulatorArgs for FullMachine {
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut machine = self.qemu_machine.arg();
        match self.specific {
            MachineType::X86_64 { smm, no_hpet } => {
                if smm {
                    machine.push(",smm=on");
                } else {
                    machine.push(",smm=off");
                }
                if no_hpet {
                    machine.push(",hpet=off");
                }
                machine.push(",vmport=off");
            }
            MachineType::AArch64 => {
                machine.push(",virtualization=on,pflash0=rom,pflash1=efivars");
            }
            MachineType::Riscv64 => {
                machine.push(",usb=on");
            }
        }
        [arg!("-machine"), oarg!(machine)]
    }
}

enum QemuMachineType {
    Qemu32,
    Pc,
    Virt,
}

impl QemuMachineType {
    fn arg(&self) -> OsString {
        match self {
            Self::Qemu32 => "q35".into(),
            Self::Pc => "pc".into(),
            Self::Virt => "virt".into(),
        }
    }
}
