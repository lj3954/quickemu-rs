use crate::{
    args::guest::GuestTweaks,
    data::{Arch, GuestOS, MacOSRelease, Machine},
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

        let (any_amd, unique_cpus) = {
            let data = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_cpu(sysinfo::CpuRefreshKind::new()));
            let unique_cpus: Box<[Box<str>]> = data
                .cpus()
                .iter()
                .dedup_by(|a, b| a.brand() == b.brand())
                .map(|c| c.brand().into())
                .collect();
            let any_amd = data.cpus().iter().any(|c| c.vendor_id() == "AuthenticAMD");
            (any_amd, unique_cpus)
        };

        let (guest_tweaks, warns) = guest.tweaks(self.arch)?;
        warnings.extend(warns);

        let cpu_type = match self.arch {
            Arch::Riscv64 { .. } => None,
            Arch::AArch64 { .. } => Some(CpuArg::Default),
            Arch::X86_64 { .. } => Some(match guest {
                GuestOS::Batocera | GuestOS::FreeBSD | GuestOS::GhostBSD | GuestOS::GenericBSD | GuestOS::FreeDOS | GuestOS::Haiku | GuestOS::Linux | GuestOS::LinuxOld | GuestOS::Solaris => {
                    CpuArg::Default
                }
                GuestOS::KolibriOS | GuestOS::ReactOS => CpuArg::Qemu32,
                GuestOS::MacOS { release } if release >= MacOSRelease::Catalina => CpuArg::Mac,
                GuestOS::MacOS { .. } => CpuArg::LegacyMac,
                GuestOS::Windows | GuestOS::WindowsServer => CpuArg::Windows,
            }),
        };

        Ok((
            Cpu {
                phys_cores: cores,
                smt,
                unique_cpus,
                any_amd,
                guest_tweaks,
                cpu_type,
            },
            warnings,
        ))
    }
}

enum CpuArg {
    Default,
    Mac,
    LegacyMac,
    Windows,
    Qemu32,
}

pub(crate) struct Cpu {
    phys_cores: usize,
    smt: bool,
    unique_cpus: Box<[Box<str>]>,
    any_amd: bool,
    guest_tweaks: GuestTweaks,
    cpu_type: Option<CpuArg>,
}

impl EmulatorArgs for Cpu {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
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
        let mut args = vec![
            Cow::Borrowed(OsStr::new("-smp")),
            Cow::Owned(format!("cores={},threads={},sockets={}", self.phys_cores, threads, sockets).into()),
        ];

        if let Some(arg) = &self.cpu_type {
            args.push(Cow::Borrowed(OsStr::new("-cpu")));
            let arg = match arg {
                CpuArg::Default => default_cpu(self.guest_tweaks.hw_virt).into(),
                CpuArg::Mac => macos_cpu_flags("Skylake-Server-v3,vendor=GenuineIntel,vmware-cpuid-freq=on"),
                CpuArg::LegacyMac => macos_legacy_cpu_flag(),
                CpuArg::Windows => format!(
                    "{},+hypervisor,+invtsc,l3-cache=on,migratable=no,hv_passthrough",
                    default_cpu(self.guest_tweaks.hw_virt)
                ),
                CpuArg::Qemu32 => "qemu32".into(),
            };
            if self.any_amd {
                args.push(Cow::Owned((arg + ",topoext").into()));
            } else {
                args.push(Cow::Owned(arg.into()));
            }
        }
        args.extend(self.guest_tweaks.qemu_args());
        args
    }
}

fn default_cpu(hw_accel: bool) -> &'static str {
    if hw_accel {
        if cfg!(target_os = "macos") {
            "host,-pdpe1gb"
        } else {
            "host"
        }
    } else {
        "max"
    }
}

fn macos_legacy_cpu_flag() -> String {
    let mut cpu_arg = "Penryn,vendor=GenuineIntel,+sse,+sse2,+ssse3,+sse4.1".to_string();
    #[cfg(target_arch = "x86_64")]
    {
        let cpuid = raw_cpuid::CpuId::new();

        if let Some(features) = cpuid.get_feature_info() {
            cpu_arg.add_features(&[
                (features.has_tsc(), "tsc"),
                (features.has_vme(), "vme"),
                (features.has_fxsave_fxstor(), "fxsr"),
                (features.has_mmx(), "mmx"),
                (features.has_clflush(), "clflush"),
                (features.has_pse36(), "pse36"),
                (features.has_pat(), "pat"),
                (features.has_cmov(), "cmov"),
                (features.has_mca(), "mca"),
                (features.has_pge(), "pge"),
                (features.has_mtrr(), "mtrr"),
                (features.has_sysenter_sysexit(), "sep"),
                (features.has_apic(), "apic"),
                (features.has_cmpxchg8b(), "cx8"),
                (features.has_mce(), "mce"),
                (features.has_pae(), "pae"),
                (features.has_msr(), "msr"),
                (features.has_pse(), "pse"),
                (features.has_de(), "de"),
                (features.has_fpu(), "fpu"),
                (features.has_cmpxchg16b(), "cx16"),
            ]);
        }
        if let Some(features) = cpuid.get_extended_processor_and_feature_identifiers() {
            cpu_arg.add_features(&[
                (features.has_64bit_mode(), "lm"),
                (features.has_execute_disable(), "nx"),
                (features.has_syscall_sysret(), "syscall"),
                (features.has_lahf_sahf(), "lahf-lm"),
            ]);
        }
    }
    cpu_arg
}

fn macos_cpu_flags(input: &str) -> String {
    let mut cpu_arg = input.to_string();
    #[cfg(target_arch = "x86_64")]
    {
        let cpuid = raw_cpuid::CpuId::new();

        if let Some(features) = cpuid.get_feature_info() {
            cpu_arg.remove_features(&[
                (features.has_aesni(), "aes"),
                (features.has_apic(), "apic"),
                (features.has_clflush(), "clflush"),
                (features.has_cmov(), "cmov"),
                (features.has_cmpxchg8b(), "cx8"),
                (features.has_cmpxchg16b(), "cx16"),
                (features.has_de(), "de"),
                (features.has_f16c(), "f16c"),
                (features.has_fma(), "fma"),
                (features.has_fxsave_fxstor(), "fxsr"),
                (features.has_mca(), "mca"),
                (features.has_mce(), "mce"),
                (features.has_mmx(), "mmx"),
                (features.has_movbe(), "movbe"),
                (features.has_msr(), "msr"),
                (features.has_mtrr(), "mtrr"),
                (features.has_pae(), "pae"),
                (features.has_pat(), "pat"),
                (features.has_pcid(), "pcid"),
                (features.has_pge(), "pge"),
                (features.has_pse(), "pse"),
                (features.has_pse36(), "pse36"),
                (features.has_popcnt(), "popcnt"),
                (features.has_rdrand(), "rdrand"),
                (features.has_sysenter_sysexit(), "sep"),
                (features.has_x2apic(), "x2apic"),
                (features.has_xsave(), "xsave"),
            ]);
        }
        if let Some(features) = cpuid.get_extended_feature_info() {
            cpu_arg.remove_features(&[
                (features.has_adx(), "adx"),
                (features.has_avx2(), "avx2"),
                (features.has_avx512f(), "avx512f"),
                (features.has_avx512bw(), "avx512bw"),
                (features.has_avx512dq(), "avx512dq"),
                (features.has_avx512cd(), "avx512cd"),
                (features.has_avx512vl(), "avx512vl"),
                (features.has_clwb(), "clwb"),
                (features.has_bmi1(), "bmi1"),
                (features.has_bmi2(), "bmi2"),
                (features.has_rep_movsb_stosb(), "erms"),
                (features.has_fsgsbase(), "fsgsbase"),
                (features.has_invpcid(), "invpcid"),
                (features.has_mpx(), "mpx"),
                (features.has_smep(), "smep"),
                (features.has_vaes(), "vaes"),
                (features.has_vpclmulqdq(), "vpclmulqdq"),
            ]);
        }
        if let Some(features) = cpuid.get_extended_processor_and_feature_identifiers() {
            cpu_arg.remove_features(&[
                (features.has_lzcnt(), "abm"),
                (features.has_lahf_sahf(), "lahf-lm"),
                (features.has_64bit_mode(), "lm"),
                (features.has_execute_disable(), "nx"),
                (features.has_syscall_sysret(), "syscall"),
                (features.has_1gib_pages() || cfg!(target_os = "macos"), "pdpe1gb"),
            ]);
        }
        if let Some(features) = cpuid.get_extended_state_info() {
            cpu_arg.remove_features(&[(features.has_xgetbv(), "xgetbv1"), (features.has_xsaveopt(), "xsaveopt")]);
        }
        if let Some(features) = cpuid.get_thermal_power_info() {
            cpu_arg.remove_features(&[(features.has_arat(), "arat")]);
        }
    }
    cpu_arg
}

trait ConditionalCpuFeatures {
    fn add_features(&mut self, features: &[(bool, &str)]);
    fn remove_features(&mut self, features: &[(bool, &str)]);
}
impl ConditionalCpuFeatures for String {
    fn add_features(&mut self, features: &[(bool, &str)]) {
        modify_features(self, features, FeatureAction::Add)
    }
    fn remove_features(&mut self, features: &[(bool, &str)]) {
        modify_features(self, features, FeatureAction::Remove)
    }
}

enum FeatureAction {
    Add,
    Remove,
}

fn modify_features(arg: &mut String, features: &[(bool, &str)], action: FeatureAction) {
    let mut modify_arg = |modification: &str, flag: &str| {
        arg.push_str(modification);
        arg.push_str(flag);
    };
    features.iter().for_each(|(has_feature, flag)| match (has_feature, &action) {
        (true, FeatureAction::Add) => modify_arg(",+", flag),
        (false, FeatureAction::Remove) => modify_arg(",-", flag),
        _ => {}
    })
}
