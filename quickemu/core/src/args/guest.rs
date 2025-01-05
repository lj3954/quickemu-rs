use std::{borrow::Cow, ffi::OsStr};

use crate::{
    data::{Arch, GuestOS},
    error::{Error, Warning},
    utils::{EmulatorArgs, QemuArg},
};

impl GuestOS {
    #[cfg(target_arch = "x86_64")]
    pub fn validate_cpu(&self) -> Result<(), Error> {
        use crate::data::MacOSRelease;

        let cpuid = raw_cpuid::CpuId::new();
        log::trace!("Testing architecture. Found CPUID: {:?}", cpuid);

        let Some(cpu_features) = cpuid.get_feature_info() else { return Ok(()) };

        if let GuestOS::MacOS { release } = self {
            if !cpu_features.has_sse41() {
                return Err(Error::Instructions("SSE4.1"));
            }
            if release >= &MacOSRelease::Ventura {
                let Some(extended_features) = cpuid.get_extended_feature_info() else { return Ok(()) };

                if !cpu_features.has_sse42() {
                    return Err(Error::Instructions("SSE4.2"));
                }
                if !extended_features.has_avx2() {
                    return Err(Error::Instructions("AVX2"));
                }
            }
        }

        Ok(())
    }
    pub(crate) fn tweaks(&self, arch: Arch) -> Result<(GuestTweaks, Vec<Warning>), Error> {
        let mut warnings = Vec::new();

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let hw_virt = arch.enable_hw_virt().unwrap_or_else(|w| {
            warnings.push(w);
            false
        });
        #[cfg(target_os = "linux")]
        let discard_lost_ticks = hw_virt && matches!(self, Self::Windows | Self::WindowsServer | Self::MacOS { .. });
        let disable_s3 = matches!(self, Self::Windows | Self::WindowsServer | Self::MacOS { .. });
        let osk = matches!(self, Self::MacOS { .. });

        Ok((
            GuestTweaks {
                #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                hw_virt,
                #[cfg(target_os = "linux")]
                discard_lost_ticks,
                disable_s3,
                osk,
            },
            warnings,
        ))
    }
}

const OSK: &[u8] = &[
    0x6f, 0x75, 0x72, 0x68, 0x61, 0x72, 0x64, 0x77, 0x6f, 0x72, 0x6b, 0x62, 0x79, 0x74, 0x68, 0x65, 0x73, 0x65, 0x77, 0x6f, 0x72, 0x64, 0x73, 0x67, 0x75, 0x61, 0x72, 0x64, 0x65, 0x64, 0x70, 0x6c,
    0x65, 0x61, 0x73, 0x65, 0x64, 0x6f, 0x6e, 0x74, 0x73, 0x74, 0x65, 0x61, 0x6c, 0x28, 0x63, 0x29, 0x41, 0x70, 0x70, 0x6c, 0x65, 0x43, 0x6f, 0x6d, 0x70, 0x75, 0x74, 0x65, 0x72, 0x49, 0x6e, 0x63,
];

pub(crate) struct GuestTweaks {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    hw_virt: bool,
    #[cfg(target_os = "linux")]
    discard_lost_ticks: bool,
    disable_s3: bool,
    osk: bool,
}

impl EmulatorArgs for GuestTweaks {
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut tweaks = Vec::new();

        #[cfg(target_os = "linux")]
        if self.hw_virt {
            tweaks.extend([Cow::Borrowed(OsStr::new("-accel")), Cow::Borrowed(OsStr::new("kvm"))]);
        }
        #[cfg(target_os = "macos")]
        if self.hw_virt {
            tweaks.extend([Cow::Borrowed(OsStr::new("-accel")), Cow::Borrowed(OsStr::new("hvf"))]);
        }
        #[cfg(target_os = "windows")]
        if self.hw_virt {
            tweaks.extend([Cow::Borrowed(OsStr::new("-accel")), Cow::Borrowed(OsStr::new("whpx"))]);
        }

        #[cfg(target_os = "linux")]
        if self.discard_lost_ticks {
            tweaks.extend([Cow::Borrowed(OsStr::new("-global")), Cow::Borrowed(OsStr::new("kvm-pit.lost_tick_policy=discard"))]);
        }

        if self.disable_s3 {
            tweaks.extend([Cow::Borrowed(OsStr::new("-global")), Cow::Borrowed(OsStr::new("ICH9-LPC.disable_s3=1"))]);
        }

        if self.osk {
            tweaks.extend([Cow::Borrowed("-device".as_ref()), Cow::Borrowed(std::str::from_utf8(OSK).unwrap().as_ref())]);
        }

        tweaks
    }
}
