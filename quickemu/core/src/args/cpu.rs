use crate::{
    data::{GuestOS, Machine},
    error::{Error, Warning},
    utils::{plural_if, ArgDisplay, EmulatorArgs, QemuArg},
};
use itertools::Itertools;
use std::{borrow::Cow, ffi::OsStr};

impl Machine {
    pub(crate) fn cpu_args(&self, guest: GuestOS) -> Result<(Cpu, Vec<Warning>), Error> {
        let mut warnings = Vec::new();

        let (cores, smt) = {
            let (physical, logical) = (num_cpus::get_physical(), num_cpus::get());

            let mut cores = if let Some(threads) = self.cpu_threads {
                if let GuestOS::MacOS { .. } = guest {
                    if !threads.is_power_of_two() {
                        let recommended = threads
                            .checked_next_power_of_two()
                            .expect("CPU cores should not overflow usize");
                        warnings.push(Warning::MacOSCorePow2(recommended));
                    }
                }
                threads.get()
            } else {
                logical
            };

            let smt = logical > physical;
            if smt {
                cores = cores.saturating_div(2);
            }
            (cores.max(1), smt)
        };

        let unique_cpus = {
            let data = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_cpu(sysinfo::CpuRefreshKind::new()));
            data.cpus()
                .iter()
                .dedup_by(|a, b| a.brand() == b.brand())
                .map(|c| c.brand().into())
                .collect()
        };

        Ok((Cpu { phys_cores: cores, smt, unique_cpus }, warnings))
    }
}

pub(crate) struct Cpu {
    phys_cores: usize,
    smt: bool,
    unique_cpus: Box<[Box<str>]>,
}

impl EmulatorArgs for Cpu {
    fn display(&self) -> Option<crate::utils::ArgDisplay> {
        let sockets = self.unique_cpus.len();
        let cores = self.phys_cores;
        let threads = if self.smt { self.phys_cores * 2 } else { self.phys_cores };
        let cpu_list = self.unique_cpus.join(", ");
        Some(ArgDisplay {
            name: Cow::Borrowed("CPU"),
            value: Cow::Owned(format!(
                "{sockets} socket{} ({cpu_list}), {cores} core{}, {threads} thread{}",
                plural_if(sockets > 1),
                plural_if(cores > 1),
                plural_if(threads > 1),
            )),
        })
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let threads = if self.smt { 2 } else { 1 };
        let sockets = self.unique_cpus.len();
        [
            Cow::Borrowed(OsStr::new("-smp")),
            Cow::Owned(format!("cores={},threads={},sockets={}", self.phys_cores, threads, sockets).into()),
        ]
    }
}
