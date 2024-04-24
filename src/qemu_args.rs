use std::ffi::OsString;
use anyhow::{anyhow, bail, Result};
use crate::{config_parse::BYTES_PER_GB, config::*};
use which::which;
use sysinfo::{System, RefreshKind, CpuRefreshKind, Cpu, Networks};
use std::path::{Path, PathBuf};

impl Args {
    pub fn into_qemu_args(self) -> Result<(OsString, Vec<OsString>)> {
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

        let publicdir: Option<String> = self.public_dir.try_into()?;

        let mut qemu_args: Vec<OsString> = Vec::with_capacity(8);
        let mut print_args: Vec<String> = Vec::with_capacity(8);

        
        cpu_ram(self.cpu_cores.0, self.cpu_cores.1, cpu_info.cpus(), self.ram, &self.guest_os)?.into_iter().add_args(&mut qemu_args, &mut print_args);
        self.network.into_args(&self.vm_name, self.ssh_port, self.port_forwards, publicdir, &self.guest_os)?.into_iter().add_args(&mut qemu_args, &mut print_args);

        let (print_boot, boot_args) = self.boot.to_args(&self.vm_dir, &self.guest_os, &self.arch)?;
        print_args.push(print_boot);
        if let Some(mut args) = boot_args {
            qemu_args.append(&mut args);
        }
        



        log::debug!("QEMU ARGS: {:?}", qemu_args);

        println!("QuickemuRS {} using {} {}.", env!("CARGO_PKG_VERSION"), qemu_bin.to_str().unwrap(), friendly_ver);

        todo!()
    }
}

trait AddToLists {
    fn add_args(self, arg_list: &mut Vec<OsString>, print_list: &mut Vec<String>);
}

impl<T, I> AddToLists for T where T: Iterator<Item=(I, Option<String>)>, I: Into<OsString>, {
    fn add_args(self, arg_list: &mut Vec<OsString>, print_list: &mut Vec<String>) {
        self.for_each(|(arg, msg)| {
            arg_list.push(arg.into());
            if let Some(msg) = msg {
                print_list.push(msg);
            }
        });
    }
}


const SECURE_BOOT_OVMF: [(&str, &str); 7] = [
    ("/usr/share/OVMF/OVMF_CODE_4M.secboot.fd", "/usr/share/OVMF/OVMF_VARS_4M.fd"),
    ("/usr/share/edk2/ovmf/OVMF_CODE.secboot.fd" ,"/usr/share/edk2/ovmf/OVMF_VARS.fd"),
    ("/usr/share/OVMF/x64/OVMF_CODE.secboot.fd", "/usr/share/OVMF/x64/OVMF_VARS.fd"),
    ("/usr/share/edk2-ovmf/OVMF_CODE.secboot.fd", "/usr/share/edk2-ovmf/OVMF_VARS.fd"),
    ("/usr/share/qemu/ovmf-x86_64-smm-ms-code.bin", "/usr/share/qemu/ovmf-x86_64-smm-ms-vars.bin"), 
    ("/usr/share/qemu/edk2-x86_64-secure-code.fd", "/usr/share/qemu/edk2-x86_64-code.fd"),
    ("/usr/share/edk2-ovmf/x64/OVMF_CODE.secboot.fd", "/usr/share/edk2-ovmf/x64/OVMF_VARS.fd"),
];
const EFI_OVMF: [(&str, &str); 8] = [
    ("/usr/share/OVMF/OVMF_CODE_4M.fd", "/usr/share/OVMF/OVMF_VARS_4M.fd"),
    ("/usr/share/edk2/ovmf/OVMF_CODE.fd", "/usr/share/edk2/ovmf/OVMF_VARS.fd"),
    ("/usr/share/OVMF/OVMF_CODE.fd", "/usr/share/OVMF/OVMF_VARS.fd"),
    ("/usr/share/OVMF/x64/OVMF_CODE.fd", "/usr/share/OVMF/x64/OVMF_VARS.fd"),
    ("/usr/share/edk2-ovmf/OVMF_CODE.fd", "/usr/share/edk2-ovmf/OVMF_VARS.fd"),
    ("/usr/share/qemu/ovmf-x86_64-4m-code.bin", "/usr/share/qemu/ovmf-x86_64-4m-vars.bin"),
    ("/usr/share/qemu/edk2-x86_64-code.fd", "/usr/share/qemu/edk2-x86_64-code.fd"),
    ("/usr/share/edk2-ovmf/x64/OVMF_CODE.fd", "/usr/share/edk2-ovmf/x64/OVMF_VARS.fd"), 
];
const AARCH64_OVMF: [(&str, &str); 1] = [
    ("/usr/share/AAVMF/AAVMF_VARS.fd", "/usr/share/AAVMF/AAVMF_CODE.fd"),
];

impl BootType {
    fn to_args(&self, vm_dir: &Path, guest_os: &GuestOS, arch: &Arch) -> Result<(String, Option<Vec<OsString>>)> {
        match (self, arch) {
            (Self::Efi { secure_boot: _ }, Arch::riscv64) => Ok(("Boot: EFI (RISC-V)".to_string(), None)),
            (Self::Legacy, Arch::x86_64) => if let GuestOS::MacOS(_) = guest_os {
                Ok(("Boot: Legacy/BIOS".to_string(), None))
            } else {
                bail!("macOS guests require EFI boot.");
            },
            (Self::Efi { secure_boot }, _) => {
                let (ovmf_code, ovmf_vars) = match guest_os {
                    GuestOS::MacOS(_) => {
                        if *secure_boot {
                            bail!("macOS guests do not support Secure Boot.");
                        }
                        let efi_code = vm_dir.join("OVMF_CODE.fd");
                        if !efi_code.exists() {
                            bail!("macOS firmware \"OVMF_CODE.fd\" could not be found.");
                        }
                        let efi_vars = ["OVMF_VARS-1024x768.fd", "OVMF_VARS-1920x1080.fd"].iter().find_map(|vars| {
                            let efi_vars = vm_dir.join(vars);
                            if efi_vars.exists() {
                                Some(efi_vars)
                            } else {
                                None
                            }
                        }).ok_or_else(|| anyhow!("macOS EFI VARS could not be found."))?;
                        (efi_code, efi_vars)
                    },
                    _ => {
                        let vm_vars = vm_dir.join("OVMF_VARS.fd");
                        let (efi_code, extra_vars) = if arch == &Arch::aarch64 {
                            find_firmware(&AARCH64_OVMF).ok_or_else(|| anyhow!("Firmware for aarch64 could not be found."))?
                        } else if *secure_boot {
                            find_firmware(&SECURE_BOOT_OVMF).ok_or_else(|| anyhow!("Secure Boot capable firmware could not be found."))?
                        } else {
                            find_firmware(&EFI_OVMF).ok_or_else(|| anyhow!("EFI firmware could not be found. Please install OVMF firmware."))?
                        };
                        let efi_code = if efi_code.is_symlink() {
                            efi_code.read_link()?
                        } else {
                            efi_code
                        };
                        if !vm_vars.exists() || !vm_vars.metadata()?.permissions().readonly() {
                            std::fs::copy(extra_vars, &vm_vars)?;
                        }
                        (efi_code, vm_vars)
                    },
                };
                let mut ovmf_code_final = OsString::from("-drive if=pflash,format=raw,unit=0,file=\"");
                ovmf_code_final.push(&ovmf_code);
                ovmf_code_final.push("\",readonly=on");
                let mut ovmf_vars_final = OsString::from("-drive if=pflash,format=raw,unit=1,file=\"");
                ovmf_vars_final.push(&ovmf_vars);
                ovmf_vars_final.push("\"");
                if arch == &Arch::aarch64 {
                    Ok(("Boot: EFI (aarch64), OVMF: ".to_string() + ovmf_code.to_str().unwrap(), Some(vec![ovmf_code_final, ovmf_vars_final])))
                } else {
                    let driver = OsString::from("-global driver=cfi.pflash01,property=secure,value=on");
                    Ok(("Boot: EFI (x86_64), OVMF: ".to_string() + ovmf_code.to_str().unwrap() + ", Secure Boot: " + if *secure_boot { "Enabled" } else { "Disabled" }, Some(vec![driver, ovmf_code_final, ovmf_vars_final])))
                }
            }
            _ => bail!("The specified combination of architecture and boot type is not currently supported."),
        }
    }
}

fn find_firmware(firmware: &[(&str, &str)]) -> Option<(PathBuf, PathBuf)> {
    firmware.iter().find_map(|(code, vars)| {
        let code = PathBuf::from(code);
        let vars = PathBuf::from(vars);
        if code.exists() && vars.exists() {
            Some((code, vars))
        } else {
            None
        }
    })
}

impl Network {
    fn into_args(self, vmname: &str, ssh: u16, port_forwards: Option<Vec<(u16, u16)>>, publicdir: Option<String>, guest_os: &GuestOS) -> Result<Vec<(String, Option<String>)>> {
        match self {
            Self::None => Ok(vec![(("-nic none".to_string()), Some("Network: Disabled".to_string()))]),
            Self::Restrict | Self::Nat => {
                let (port_forwards, port_forward_msg) = match port_forwards {
                    Some(forwards) => {
                        let data: (Vec<String>, Vec<String>) = forwards.iter()
                        .map(|(host, guest)| (format!(",hostfwd=tcp::{}-:{},hostfwd=udp::{}-:{}", host, guest, host, guest), format!("{} => {}", host, guest)))
                        .unzip();
                        (Some(data.0.join("")), Some(format!("Port forwards: {}", data.1.join(", "))))
                    },
                    None => (None, None),
                };

                let net = {
                    let samba = which("smbd").ok().map(|_| format!(",smb={}", publicdir.unwrap_or_default())).unwrap_or_default();
                    let ssh = format!(",hostfwd=tcp::{}-:22", ssh);
                    format!("user,hostname={vmname}{ssh}{}{samba}", port_forwards.unwrap_or_default())
                };

                let device = guest_os.net_device();
                let device_arg = format!("-device {},netdev=nic", device);
                let netdev = format!("-netdev {}{},id=nic", net, if self == Self::Restrict { ",restrict=y" } else { "" });

                let msg = format!("Network: {} ({}), SSH (On host): ssh user@localhost -p {ssh}", if self == Self::Restrict { "Restricted" } else { "User" }, device);
                Ok(vec![(netdev, Some(msg)), (device_arg, port_forward_msg)])
            },
            Self::Bridged { bridge, mac_addr } => {
                let network = Networks::new_with_refreshed_list();
                if !network.contains_key(&bridge) {
                    bail!("Network interface {} could not be found.", bridge);
                }
                let mac_addr = mac_addr.and_then(|mac| format!(",mac={}", mac).into()).unwrap_or_default();
                Ok(vec![(format!("-nic bridge,br={}{}", bridge, mac_addr), Some(format!("Network: Bridged ({})", bridge)))])
            },
        }
    }
}

impl TryInto<Option<String>> for PublicDir {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Option<String>> {
        Ok(match self {
            Self::None => None,
            Self::Default => {
                let public = dirs::public_dir();
                if public != dirs::home_dir() {
                    public.map(|dir| dir.to_string_lossy().to_string())
                } else {
                    None
                }
            },
            Self::Custom(dir) => if PathBuf::from(&dir).exists() {
                Some(dir)
            } else {
                bail!("Chosen public directory {} does not exist.", dir)
            },
        })
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
        GuestOS::MacOS(release) => !matches!(release, MacOSRelease::HighSierra | MacOSRelease::Mojave | MacOSRelease::Catalina),
        _ => true,
    };

    let ram_arg = format!("-m {} {}", ram, if balloon { "-device virtio-balloon" } else { "" });

    Ok([(core_arg, Some(format!("Using {}{}, and {} GB of RAM.", socket_text, core_text, ram as f64 / BYTES_PER_GB as f64))), (ram_arg, None)])
}

#[cfg(target_arch = "x86_64")]
impl GuestOS {
    fn validate_cpu(&self) -> Result<()> {
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
    
    fn net_device(&self) -> &'static str {
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
}
