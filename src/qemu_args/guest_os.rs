use anyhow::{anyhow, bail, Result};
use crate::config::{GuestOS, Arch, MacOSRelease};
use sysinfo::{System, RefreshKind, CpuRefreshKind};
use crate::config_parse::BYTES_PER_GB;

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

        if let GuestOS::MacOS(release) = self {
            if matches!(release, MacOSRelease::Ventura | MacOSRelease::Sonoma) {
                if let Some(extended_features) = cpuid.get_extended_feature_info() {
                    if !(cpu_features.has_sse41() || extended_features.has_avx2()) {
                        bail!("macOS releases Ventura and newer require a CPU which supports AVX2 and SSE4.1.");
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
            Self::MacOS(release) => match release {
                MacOSRelease::BigSur | MacOSRelease::Monterey | MacOSRelease::Ventura | MacOSRelease::Sonoma => "virtio-net",
                _ => "vmxnet3",
            },
            Self::Linux | Self::Solaris | Self::GhostBSD => "virtio-net",
            _ => "rtl8139",
        }
    }

    pub fn cpu_argument(&self, arch: &Arch) -> String {
        let cpu = match arch {
            Arch::aarch64 | Arch::riscv64 => "max".to_string(),
            Arch::x86_64 => {
                let is_amd = || System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::new())).cpus()[0].vendor_id().contains("AuthenticAMD");
                let default_cpu = || if arch.matches_host() {
                    "host".to_string()
                } else {
                    "max".to_string()
                };
                match self {
                    Self::Batocera | Self::FreeBSD | Self::GhostBSD | Self::FreeDOS | Self::Haiku | Self::Linux | Self::Solaris => if is_amd() {
                        default_cpu() + ",topoext"
                    } else {
                        default_cpu()
                    },
                    Self::KolibriOS | Self::ReactOS => if is_amd() {
                        "qemu32,topoext".to_string()
                    } else {
                        "qemu32".to_string()
                    },
                    Self::MacOS(release) if release >= &MacOSRelease::Ventura => "Broadwell-noTSX-IBRS,vendor=GenuineIntel,+sse,+sse4.1,+avx,+avx2,+hypervisor,+popcnt,+aes,+xsave,+xsavec,+xsaveopt,+xgetbv1,+bmi1,+bmi2,+smep,+fma,+movbe,+invtsc".to_string(),
                    Self::MacOS(_) => "Penryn,vendor=GenuineIntel,+aes,+avx,+bmi1,+bmi2,+fma,+hypervisor,+invtsc,+kvm_pv_eoi,+kvm_pv_unhalt,+popcnt,+ssse3,+sse4.2,vmware-cpuid-freq=on,+xsave,+xsaveopt,check".to_string(),
                    Self::Windows | Self::WindowsServer => default_cpu() + ",+hypervisor,+invtsc,l3-cache=on,migratable=no,hv_passthrough",
                }
            }
        };
        if arch.matches_host() {
            cpu + ",kvm=on"
        } else {
            cpu
        }
    }       

    pub fn disk_size(&self) -> u64 {
        match self {
            Self::Windows | Self::WindowsServer => 64 * BYTES_PER_GB,
            Self::MacOS(_) => 96 * BYTES_PER_GB,
            Self::ReactOS | Self::KolibriOS => 16 * BYTES_PER_GB,
            _ => 32 * BYTES_PER_GB,
        }
    }
}
