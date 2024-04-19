use std::ffi::OsString;
use anyhow::{anyhow, bail, Result};
use crate::{config_parse::BYTES_PER_GB, config::*};
use which::which;
use sysinfo::{System, RefreshKind, CpuRefreshKind, Cpu};

impl Args {
    pub fn to_qemu_args(&self) -> Result<(OsString, Vec<String>)> {
        let qemu_bin = match &self.arch {
            Arch::x86_64 => "qemu-system-x86_64",
            Arch::aarch64 => "qemu-system-aarch64",
            Arch::riscv64 => "qemu-system-riscv64",
        };
        let qemu_bin = which(qemu_bin).map_err(|_| anyhow!("Could not find QEMU binary: {}. Please make sure QEMU is installed on your system.", qemu_bin))?;

        let qemu_version = std::process::Command::new(&qemu_bin).arg("--version").output()?;
        let friendly_ver = std::str::from_utf8(&qemu_version.stdout)?
            .split_whitespace()
            .nth(3)
            .ok_or_else(|| anyhow::anyhow!("Failed to get QEMU version."))?;

        if friendly_ver[0..1].parse::<u8>()? < 6 {
            bail!("QEMU version 6.0.0 or higher is required. Found version {}.", friendly_ver);
        }
        
        let cpu_info = System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::new()));

        // (x86_64 only) Determine whether CPU has the necessary features. VMX support is checked
        // for all operating systems, and macOS checks SSE4.1, and AVX2 for Ventura and newer.
        #[cfg(target_arch = "x86_64")]
        if self.arch == Arch::x86_64 {
            self.guest_os.validate_cpu()?;
        }

        let mut qemu_args: Vec<(String, Option<String>)> = Vec::with_capacity(8);


        qemu_args.extend(cpu_ram(self.cpu_cores.0, self.cpu_cores.1, cpu_info.cpus(), self.ram, &self.guest_os)?);
        


            

        println!("QuickemuRS {} using {} {}.", env!("CARGO_PKG_VERSION"), qemu_bin.to_str().unwrap(), friendly_ver);

        todo!()
    }
}

fn cpu_ram(cores: usize, threads: bool, cpu_info: &[sysinfo::Cpu], ram: u64, guest_os: &GuestOS) -> Result<[(String, Option<String>); 2]> {
    if ram < 4 * (1024 * 1024 * 1024) {
        if let GuestOS::MacOS(_) | GuestOS::Windows | GuestOS::WindowsServer = guest_os {
            bail!("{} guests require at least 4GB of RAM.", guest_os);
        }
    }

    let mut cpus = cpu_info.iter()
        .map(|cpu: &Cpu| cpu.brand())
        .collect::<Vec<&str>>();
    // CPUs should already be in a sorted order. remove duplicates.
    cpus.dedup();

    let sockets = cpus.len();
    let socket_text = match sockets {
        1 => "".to_string(),
        _ => format!(" {} sockets,", sockets), 
    };

    let (core_text, core_arg) = if cores > 1 && threads {
        (format!("{} core{} and {} threads", cores / 2, if cores > 2 { "s" } else { "" }, cores),
        format!("-smp cores={},threads=2,sockets={}", cores / 2, sockets))
    } else {
        (format!("{} core{}", cores, if cores > 1 { "s" } else { "" }),
        format!("-smp cores={},threads=1,sockets={}", cores, sockets))
    };

    let balloon = match guest_os {
        GuestOS::MacOS(release) => match release {
            MacOSRelease::HighSierra | MacOSRelease::Mojave | MacOSRelease::Catalina => false,
            _ => true,
        },
        _ => true,
    };

    let ram_arg = format!("-m {} {}", ram, if balloon { "-device virtio-balloon" } else { "" });

    Ok([(core_arg, Some(format!("Using {}{}, and {} GB of RAM.", socket_text, core_text, ram as f64 / BYTES_PER_GB as f64))), (ram_arg, None)])
}

#[cfg(target_arch = "x86_64")]
impl GuestOS {
    pub fn validate_cpu(&self) -> Result<()> {
        let cpuid = raw_cpuid::CpuId::new();
        log::debug!("Testing architecture. Found CPUID: {:?}", cpuid);
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
        if !cpu_features.has_vmx() {
            bail!("CPU Virtualization{} is required for x86_64 guests. Please enable it in your BIOS.", virtualization_type);
        }

        match self {
            GuestOS::MacOS(release) => match release {
                MacOSRelease::Ventura | MacOSRelease::Sonoma => {
                    let extended_features = cpuid.get_extended_feature_info().ok_or_else(|| anyhow!("Could not determine whether your CPU supports AVX2."))?;
                    if !(cpu_features.has_sse41() || extended_features.has_avx2()) {
                        bail!("macOS releases Ventura and newer require a CPU which supports AVX2 and SSE4.1.");
                    }
                },
                _ => if !cpu_features.has_sse41() {
                    bail!("macOS requires a CPU which supports SSE4.1.");
                },
            },
            _ => (),
        }
        Ok(())
    }
}
