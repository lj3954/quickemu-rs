use crate::{data::Arch, error::Warning};
use raw_cpuid::{CpuId, CpuIdReader};
use std::path::Path;

impl Arch {
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    pub(crate) fn enable_hw_virt(&self) -> Result<bool, Warning> {
        if !self.matches_host() {
            return Ok(false);
        }
        #[cfg(target_arch = "x86_64")]
        #[cfg(not(target_os = "linux"))]
        {
            let cpuid = CpuId::new();
            let has_vmx = cpuid.get_feature_info().map_or(true, |f| f.has_vmx());
            let has_svm = cpuid
                .get_extended_processor_and_feature_identifiers()
                .map_or(true, |f| f.has_svm());
            if !has_vmx && !has_svm {
                return Err(Warning::HwVirt(query_virt_type(cpuid)));
            }
        }
        #[cfg(target_os = "linux")]
        if !Path::new("/dev/kvm").exists() {
            #[cfg(target_arch = "x86_64")]
            return Err(Warning::HwVirt(query_virt_type(CpuId::new())));
            #[cfg(not(target_arch = "x86_64"))]
            return Err(Warning::HwVirt(""));
        }
        Ok(true)
    }
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn matches_host(&self) -> bool {
        match self {
            Self::X86_64 { .. } => cfg!(target_arch = "x86_64"),
            Self::AArch64 { .. } => cfg!(target_arch = "aarch64"),
            Self::Riscv64 { .. } => cfg!(target_arch = "riscv64"),
        }
    }
}

fn query_virt_type<R: CpuIdReader>(cpuid_reader: CpuId<R>) -> &'static str {
    cpuid_reader.get_vendor_info().map_or("", |v| match v.as_str() {
        "GenuineIntel" => " (VT-x)",
        "AuthenticAMD" => " (AMD-V)",
        _ => "",
    })
}
