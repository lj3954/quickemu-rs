use raw_cpuid::{CpuId, CpuIdReader};

use crate::qemu_args::{Arch, GuestOS};
use once_cell::sync::Lazy;
use std::{ffi::OsString, path::Path};

pub static SUPPORTS_HW_VIRT: Lazy<bool> = Lazy::new(|| {
    #[cfg(target_arch = "x86_64")]
    #[cfg(not(target_os = "linux"))]
    {
        let cpuid = CpuId::new();
        let has_vmx = cpuid.get_feature_info().map_or(false, |f| f.has_vmx());
        let has_svm = cpuid
            .get_extended_processor_and_feature_identifiers()
            .map_or(false, |f| f.has_svm());
        if !has_vmx && !has_svm {
            return virt_warning(query_virt_type(cpuid));
        }
    }
    #[cfg(target_os = "linux")]
    if !Path::new("/dev/kvm").exists() {
        return virt_warning(query_virt_type(CpuId::new()));
    }
    true
});

impl Arch {
    pub fn enable_hw_virt(&self) -> bool {
        self.matches_host() && *SUPPORTS_HW_VIRT
    }

    fn matches_host(&self) -> bool {
        match self {
            Self::x86_64 => cfg!(target_arch = "x86_64"),
            Self::aarch64 => cfg!(target_arch = "aarch64"),
            Self::riscv64 => cfg!(target_arch = "riscv64"),
        }
    }

    pub fn machine_type(&self, guest_os: &GuestOS) -> OsString {
        match self {
            Self::x86_64 => {
                let machine_type = match guest_os {
                    GuestOS::Windows | GuestOS::WindowsServer | GuestOS::MacOS { .. } => "q35,hpet=off",
                    GuestOS::FreeDOS | GuestOS::Batocera | GuestOS::Haiku | GuestOS::Solaris | GuestOS::ReactOS | GuestOS::KolibriOS => "pc",
                    _ => "q35",
                };

                #[cfg(not(target_os = "macos"))]
                let smm = match guest_os {
                    GuestOS::Windows | GuestOS::WindowsServer | GuestOS::FreeDOS => "on",
                    _ => "off",
                };

                #[cfg(target_os = "macos")]
                let smm = "off";

                let mut machine = OsString::from(machine_type);
                machine.push(",smm=");
                machine.push(smm);
                machine.push(",vmport=off");
                machine
            }
            Self::aarch64 => "virt,virtualization=on,pflash0=rom,pflash1=efivars".into(),
            Self::riscv64 => "virt,usb=on".into(),
        }
    }
}

fn query_virt_type<R: CpuIdReader>(cpuid_reader: CpuId<R>) -> Option<&'static str> {
    cpuid_reader.get_vendor_info().and_then(|v| match v.as_str() {
        "GenuineIntel" => Some(" (VT-x)"),
        "AuthenticAMD" => Some(" (AMD-V)"),
        _ => None,
    })
}

fn virt_warning(virt_type: Option<&'static str>) -> bool {
    log::warn!(
        "Virtualization{} is not enabled on your CPU. Falling back to software virtualization, performance will be degraded.",
        virt_type.unwrap_or_default()
    );
    false
}
