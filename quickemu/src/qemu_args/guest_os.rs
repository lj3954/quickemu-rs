use crate::config::{Arch, GuestOS, MacOSRelease};
use crate::config_parse::BYTES_PER_GB;
use anyhow::{anyhow, bail, Result};
use std::ffi::OsString;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

const OSK: &[u8] = &[
    0x6f, 0x75, 0x72, 0x68, 0x61, 0x72, 0x64, 0x77, 0x6f, 0x72, 0x6b, 0x62, 0x79, 0x74, 0x68, 0x65, 0x73, 0x65, 0x77, 0x6f, 0x72, 0x64, 0x73, 0x67, 0x75, 0x61, 0x72, 0x64, 0x65, 0x64, 0x70, 0x6c,
    0x65, 0x61, 0x73, 0x65, 0x64, 0x6f, 0x6e, 0x74, 0x73, 0x74, 0x65, 0x61, 0x6c, 0x28, 0x63, 0x29, 0x41, 0x70, 0x70, 0x6c, 0x65, 0x43, 0x6f, 0x6d, 0x70, 0x75, 0x74, 0x65, 0x72, 0x49, 0x6e, 0x63,
];

impl GuestOS {
    #[cfg(target_arch = "x86_64")]
    pub fn validate_cpu(&self) -> Result<()> {
        let cpuid = raw_cpuid::CpuId::new();
        log::trace!("Testing architecture. Found CPUID: {:?}", cpuid);
        let virtualization_type = match cpuid.get_vendor_info() {
            Some(vendor_info) => match vendor_info.as_str() {
                "GenuineIntel" => " (VT-x)",
                "AuthenticAMD" => " (AMD-V)",
                _ => "",
            },
            None => "",
        };

        let cpu_features = cpuid
            .get_feature_info()
            .ok_or_else(|| anyhow!("Could not determine whether your CPU supports the necessary instructions."))?;
        let extended_identifiers = cpuid
            .get_extended_processor_and_feature_identifiers()
            .ok_or_else(|| anyhow!("Could not determine whether your CPU supports the necessary instructions."))?;
        if !(cpu_features.has_vmx() || extended_identifiers.has_svm()) {
            bail!("CPU Virtualization{virtualization_type} is required for x86_64 guests. Please enable it in your BIOS.",);
        }

        if let GuestOS::MacOS { release } = self {
            if matches!(release, MacOSRelease::Ventura | MacOSRelease::Sonoma) {
                if let Some(extended_features) = cpuid.get_extended_feature_info() {
                    if !(cpu_features.has_sse42() || extended_features.has_avx2()) {
                        bail!("macOS releases Ventura and newer require a CPU which supports AVX2 and SSE4.2.");
                    }
                } else {
                    bail!("Could not determine whether your CPU supports AVX2.");
                }
            } else if !cpu_features.has_sse41() {
                bail!("macOS requires a CPU which supports SSE4.1.");
            }
        }

        Ok(())
    }

    pub fn net_device(&self) -> &'static str {
        match self {
            Self::Batocera | Self::FreeDOS | Self::Haiku => "rtl8139",
            Self::ReactOS => "e1000",
            Self::MacOS { release } => match release {
                MacOSRelease::BigSur | MacOSRelease::Monterey | MacOSRelease::Ventura | MacOSRelease::Sonoma => "virtio-net",
                _ => "vmxnet3",
            },
            Self::Linux | Self::LinuxOld | Self::Solaris | Self::GhostBSD => "virtio-net",
            _ => "rtl8139",
        }
    }

    pub fn cpu_argument(&self, arch: &Arch) -> Option<String> {
        let default_cpu = || {
            if arch.matches_host() {
                if cfg!(target_os = "macos") {
                    "host,-pdpe1gb".to_string()
                } else {
                    "host".to_string()
                }
            } else {
                "max".to_string()
            }
        };
        let cpu_info = System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::new()));
        let vendor = cpu_info.cpus()[0].vendor_id();
        Some(match arch {
            Arch::riscv64 => return None,
            Arch::aarch64 => default_cpu(),
            Arch::x86_64 => {
                let cpu_arg = match self {
                    Self::Batocera | Self::FreeBSD | Self::GhostBSD | Self::FreeDOS | Self::Haiku | Self::Linux | Self::LinuxOld | Self::Solaris => default_cpu(),
                    Self::KolibriOS | Self::ReactOS => "qemu32".to_string(),
                    Self::MacOS { release } => {
                        if release >= &MacOSRelease::Ventura {
                            macos_cpu_flags("Skylake-Server-v3,vendor=GenuineIntel,vmware-cpuid-freq=on")
                        } else {
                            macos_legacy_cpu_flag()
                        }
                    }
                    Self::Windows | Self::WindowsServer => default_cpu() + ",+hypervisor,+invtsc,l3-cache=on,migratable=no,hv_passthrough",
                };

                if vendor == "AuthenticAMD" {
                    cpu_arg + ",topoext"
                } else {
                    cpu_arg
                }
            }
        })
    }

    pub fn disk_size(&self) -> u64 {
        match self {
            Self::Windows | Self::WindowsServer => 64 * BYTES_PER_GB,
            Self::MacOS { .. } => 96 * BYTES_PER_GB,
            Self::ReactOS | Self::KolibriOS => 16 * BYTES_PER_GB,
            _ => 32 * BYTES_PER_GB,
        }
    }

    pub fn guest_tweaks(&self) -> Option<Vec<OsString>> {
        match self {
            Self::MacOS { .. } => {
                let mut osk = OsString::from("isa-applesmc,osk=");
                osk.push(String::from_utf8_lossy(OSK).to_string());
                Some(vec![
                    "-global".into(),
                    "kvm-pit.lost_tick_policy=discard".into(),
                    "-global".into(),
                    "ICH9-LPC.disable_s3=1".into(),
                    "-device".into(),
                    osk,
                ])
            }
            Self::Windows | Self::WindowsServer => Some(vec![
                "-global".into(),
                "kvm-pit.lost_tick_policy=discard".into(),
                "-global".into(),
                "ICH9-LPC.disable_s3=1".into(),
            ]),
            _ => None,
        }
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
    features.iter().for_each(|(has_feature, flag)| match (has_feature, &action) {
        (true, FeatureAction::Add) => {
            arg.push_str(",+");
            arg.push_str(flag);
        }
        (false, FeatureAction::Remove) => {
            arg.push_str(",-");
            arg.push_str(flag);
        }
        _ => {}
    })
}
