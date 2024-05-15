use anyhow::{anyhow, bail, Result};
use crate::config::{GuestOS, Arch, MacOSRelease};
use sysinfo::{System, RefreshKind, CpuRefreshKind};
use crate::config_parse::BYTES_PER_GB;
use std::ffi::OsString;

const OSK: &[u8] = &[0x6f, 0x75, 0x72, 0x68, 0x61, 0x72, 0x64, 0x77, 0x6f, 0x72, 0x6b, 0x62, 0x79, 0x74, 0x68, 0x65, 0x73, 0x65, 0x77, 0x6f, 0x72, 0x64, 0x73, 0x67, 0x75, 0x61, 0x72, 0x64, 0x65, 0x64, 0x70, 0x6c, 0x65, 0x61, 0x73, 0x65, 0x64, 0x6f, 0x6e, 0x74, 0x73, 0x74, 0x65, 0x61, 0x6c, 0x28, 0x63, 0x29, 0x41, 0x70, 0x70, 0x6c, 0x65, 0x43, 0x6f, 0x6d, 0x70, 0x75, 0x74, 0x65, 0x72, 0x49, 0x6e, 0x63];

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
        
        let cpu_features = cpuid.get_feature_info()
            .ok_or_else(|| anyhow!("Could not determine whether your CPU supports the necessary instructions."))?;
        let extended_identifiers = cpuid.get_extended_processor_and_feature_identifiers()
            .ok_or_else(|| anyhow!("Could not determine whether your CPU supports the necessary instructions."))?;
        if !(cpu_features.has_vmx() || extended_identifiers.has_svm()) {
            bail!("CPU Virtualization{} is required for x86_64 guests. Please enable it in your BIOS.", virtualization_type);
        }

        if let GuestOS::MacOS { release} = self {
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
        let default_cpu = || if arch.matches_host() {
            if cfg!(target_os = "macos") {
                "host,-pdpe1gb".to_string()
            } else {
                "host".to_string()
            }
        } else {
            "max".to_string()
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
                        if vendor == "GenuineIntel" {
                            default_cpu()
                        } else if release >= &MacOSRelease::Catalina {
                            macos_cpu_flags("Haswell-v4,vendor=GenuineIntel,+avx,+avx2,+sse,+sse2,+sse3,+sse4.2,vmware-cpuid-freq=on")
                        } else {
                            macos_cpu_flags("Penryn,vendor=GenuineIntel,+avx,+sse,+sse2,+sse3,+sse4.1,vmware-cpuid-freq=on")
                        }
                    },
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
            Self::MacOS {..} => 96 * BYTES_PER_GB,
            Self::ReactOS | Self::KolibriOS => 16 * BYTES_PER_GB,
            _ => 32 * BYTES_PER_GB,
        }
    }

    pub fn guest_tweaks(&self) -> Option<Vec<OsString>> {
        match self {
            Self::MacOS {..} => {
                let mut osk = OsString::from("isa-applesmc,osk=");
                osk.push(String::from_utf8_lossy(OSK).to_string());
                Some(vec!["-global".into(), "kvm-pit.lost_tick_policy=discard".into(), "-global".into(), "ICH9-LPC.disable_s3=1".into(), "-device".into(), osk])
            },
            Self::Windows | Self::WindowsServer => Some(vec!["-global".into(), "kvm-pit.lost_tick_policy=discard".into(), "-global".into(), "ICH9-LPC.disable_s3=1".into()]),
            _ => None,
        }
    }
}
fn macos_cpu_flags(input: &str) -> String {
    let mut cpu_arg = input.to_string();
    #[cfg(target_arch = "x86_64")] {
        let cpuid = raw_cpuid::CpuId::new();

        if let Some(features) = cpuid.get_feature_info() {
            [(features.has_aesni(), ",aes"),
            (features.has_cmpxchg8b(), ",cx8"),
            (features.has_eist(), ",eist"),
            (features.has_f16c(), ",f16c"),
            (features.has_fma(), ",fma"),
            (features.has_mmx(), ",mmx"),
            (features.has_movbe(), ",movbe"),
            (features.has_popcnt(), ",popcnt"),
            (features.has_xsave(), ",xsave")]
            .iter().for_each(|(has_feature, flag)| if *has_feature { cpu_arg.push_str(flag) });
        }
        if let Some(features) = cpuid.get_extended_feature_info() {
            [(features.has_bmi1(), ",abm,bmi1"),
            (features.has_bmi2(), ",bmi2"),
            (features.has_adx(), ",adx"),
            (features.has_mpx(), ",mpx"),
            (features.has_smep(), ",smep"),
            (features.has_vaes(), ",vaes"),
            (features.has_av512vbmi2(), "vbmi2"),
            (features.has_vpclmulqdq(), ",vpclmulqdq")]
            .iter().for_each(|(has_feature, flag)| if *has_feature { cpu_arg.push_str(flag) });
        }
        if let Some(features) = cpuid.get_extended_processor_and_feature_identifiers() {
            if features.has_data_access_bkpt_extension() {
                cpu_arg.push_str(",amd-ssbd");
            }
            #[cfg(not(target_os = "macos"))]
            if features.has_1gib_pages() {
                cpu_arg.push_str(",pdpe1gb");
            }
        }
        if let Some(features) = cpuid.get_advanced_power_mgmt_info() {
            if features.has_invariant_tsc() {
                cpu_arg.push_str(",invtsc");
            }
        }
        if let Some(features) = cpuid.get_extended_state_info() {
            [(features.has_xgetbv(), ",xgetbv1"),
            (features.has_xsaveopt(), ",xsaveopt")]
            .iter().for_each(|(has_feature, flag)| if *has_feature { cpu_arg.push_str(flag) });
        }
    }
    cpu_arg
}
