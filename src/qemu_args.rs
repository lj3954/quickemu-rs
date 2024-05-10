mod guest_os;
mod images;
mod display;
mod arch;

use std::ffi::OsString;
use std::{io::Write, fs::{OpenOptions, create_dir}};
use std::process::{Stdio, Command};
use anyhow::{anyhow, bail, Result};
use crate::{config_parse::BYTES_PER_GB, config::*};
use which::which;
use sysinfo::{System, Cpu, Networks};
use std::path::{Path, PathBuf};

impl Args {
    pub fn into_qemu_args(self) -> Result<(OsString, Vec<OsString>)> {
        let qemu_bin = match &self.arch {
            Arch::x86_64 => "qemu-system-x86_64",
            Arch::aarch64 => "qemu-system-aarch64",
            Arch::riscv64 => "qemu-system-riscv64",
        };
        let qemu_bin = which(qemu_bin).map_err(|_| anyhow!("Could not find QEMU binary: {}. Please make sure QEMU is installed on your system.", qemu_bin))?;

        if !self.vm_dir.exists() {
            create_dir(&self.vm_dir).map_err(|e| anyhow!("Could not create VM directory: {:?}", e))?;
        }

        let qemu_version = Command::new(&qemu_bin).stdout(Stdio::piped()).arg("--version").spawn()?;
        let smartcard = if matches!(self.arch, Arch::x86_64 | Arch::riscv64) {
            Some(Command::new(&qemu_bin).arg("-chardev").arg("spicevmc,id=ccid,name=").stderr(Stdio::piped()).spawn()?)
        } else {
            None
        };
            
        // (x86_64 only) Determine whether CPU has the necessary features. VMX support is checked
        // for all operating systems, and macOS checks SSE4.1, and AVX2 for Ventura and newer.
        #[cfg(target_arch = "x86_64")]
        if self.arch == Arch::x86_64 {
            self.guest_os.validate_cpu()?;
        }

        let publicdir: Option<OsString> = self.public_dir.try_into()?;

        let mut qemu_args: Vec<OsString> = basic_args(&self.vm_name, &self.vm_dir, &self.guest_os, &self.arch);
        let mut print_args: Vec<String> = Vec::with_capacity(16);
        qemu_args.reserve(32);
        
        qemu_args.extend(self.display.audio_arg());
        cpucores_ram(self.cpu_cores.0, self.cpu_cores.1, &self.system, self.ram, &self.guest_os)?.add_args(&mut qemu_args, &mut print_args);
        if let Some(arg) = self.guest_os.cpu_argument(&self.arch) {
            qemu_args.extend(["-cpu".into(), arg.into()]);
        }
        self.network.into_args(&self.vm_name, self.ssh_port, self.port_forwards, publicdir.as_ref(), &self.guest_os)?.add_args(&mut qemu_args, &mut print_args);
        self.display.display_args(&self.guest_os, &self.arch, self.resolution, self.fullscreen, self.accelerated)?.add_args(&mut qemu_args, &mut print_args);
        self.sound_card.to_args().add_args(&mut qemu_args, &mut print_args);
        
        if self.tpm {
            let (args, print) = tpm_args(&self.vm_dir, &self.vm_name)?;
            qemu_args.extend(args);
            print_args.push(print);
        }

        let (print_boot, boot_args) = self.boot.to_args(&self.vm_dir, &self.guest_os, &self.arch)?;
        print_args.push(print_boot);
        if let Some(mut args) = boot_args {
            qemu_args.append(&mut args);
        }
        
        images::image_args(&self.vm_dir, (self.image_file, self.fixed_iso, self.floppy), self.disk_img, self.disk_size, &self.guest_os, &self.prealloc, self.status_quo)?.add_args(&mut qemu_args, &mut print_args);
        self.usb_controller.to_args(&self.guest_os, smartcard, self.usb_devices)?.add_args(&mut qemu_args, &mut print_args);
        self.keyboard.to_args().add_args(&mut qemu_args, &mut print_args);

        if let Some(layout) = self.keyboard_layout {
            qemu_args.extend(["-k".into(), layout.into()]);
        }
        if self.braille {
            qemu_args.extend(["-chardev".into(), "braille,id=brltty".into(), "-device".into(), "usb-braille,id=usbbrl,chardev=brltty".into()]);
        }
        if let Some(publicdir) = publicdir {
            publicdir_args(&publicdir, &self.guest_os)?.add_args(&mut qemu_args, &mut print_args);
        }
        self.mouse.to_args().add_args(&mut qemu_args, &mut print_args);
        self.monitor.to_args("monitor")?.add_args(&mut qemu_args, &mut print_args);
        self.serial.to_args("serial")?.add_args(&mut qemu_args, &mut print_args);

        if let Some(args) = self.extra_args {
            qemu_args.extend(args.into_iter().map(|arg| arg.into()).collect::<Vec<OsString>>());
        }
        


        log::debug!("QEMU ARGS: {:?}", qemu_args);

        let qemu_version = qemu_version.wait_with_output()?;
        let friendly_ver = std::str::from_utf8(&qemu_version.stdout)?
            .split_whitespace()
            .nth(3)
            .ok_or_else(|| anyhow::anyhow!("Failed to get QEMU version."))?;

        if friendly_ver[0..1].parse::<u8>()? < 7 {
            bail!("QEMU version 7.0.0 or higher is required. Found version {}.", friendly_ver);
        }

        println!("QuickemuRS {} using {} {}.", env!("CARGO_PKG_VERSION"), qemu_bin.to_string_lossy(), friendly_ver);
        print_args.iter().for_each(|arg| println!(" - {}", arg));

        Ok((qemu_bin.into_os_string(), qemu_args))
    }
}

trait AddToLists {
    fn add_args(self, arg_list: &mut Vec<OsString>, print_list: &mut Vec<String>);
}

impl<T> AddToLists for (Vec<T>, Option<Vec<String>>) where T: Into<OsString> {
    fn add_args(self, arg_list: &mut Vec<OsString>, print_list: &mut Vec<String>) {
        arg_list.extend(self.0.into_iter().map(|arg| arg.into()));
        if let Some(msgs) = self.1 {
            print_list.extend(msgs);
        }
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
    ("/usr/share/AAVMF/AAVMF_CODE.fd", "/usr/share/AAVMF/AAVMF_VARS.fd"),
];

impl BootType {
    fn to_args(&self, vm_dir: &Path, guest_os: &GuestOS, arch: &Arch) -> Result<(String, Option<Vec<OsString>>)> {
        match (self, arch) {
            (Self::Efi { secure_boot: _ }, Arch::riscv64) => {
                let bios_dirs = [&vm_dir.join("boot"), vm_dir];
                let bios = bios_dirs.into_iter().filter_map(|directory| {
                    directory.read_dir().ok()?.filter_map(|file| {
                        let path = file.ok()?.path();
                        if path.extension()? == "bin" {
                            Some(path)
                        } else {
                            None
                        }
                    }).collect::<Vec<PathBuf>>().into()
                }).flatten().collect::<Vec<PathBuf>>();
                match bios.len() {
                    0 => Ok(("Boot: EFI (RISC-V). \x1b[31mWARNING\x1b[0m: Could not find bootloader in your VM directory. VM may fail to boot. Read more: {PLACEHOLDER}".to_string(), None)),
                    1 => {
                        let bios = &bios[0];
                        Ok(("Boot: EFI (RISC-V), Bootloader: ".to_string() + &bios.to_string_lossy(), Some(vec!["-kernel".into(), bios.into()])))
                    },
                    _ => bail!("Could not determine the correct RISC-V bootloader. Please ensure that there are not multiple `.bin` files in your VM directory."),
                }
            },
            (Self::Legacy, Arch::x86_64) => if let GuestOS::MacOS(_) = guest_os {
                bail!("macOS guests require EFI boot.");
            } else {
                Ok(("Boot: Legacy/BIOS".to_string(), None))
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
                        let efi_code = efi_code.canonicalize()?;
                        if !vm_vars.exists() || vm_vars.metadata()?.permissions().readonly() {
                            std::fs::copy(extra_vars, &vm_vars)?;
                        }
                        (efi_code, vm_vars)
                    },
                };
                if arch == &Arch::aarch64 {
                    let mut aavmf_code_final = OsString::from("node-name=rom,driver=file,filename=");
                    aavmf_code_final.push(&ovmf_code);
                    aavmf_code_final.push(",read-only=true");
                    let mut aavmf_vars_final = OsString::from("node-name=efivars,driver=file,filename=");
                    aavmf_vars_final.push(&ovmf_vars);
                    Ok(("Boot: EFI (aarch64), OVMF: ".to_string() + ovmf_code.to_str().unwrap(), Some(vec!["-blockdev".into(), aavmf_code_final, "-blockdev".into(), aavmf_vars_final])))
                } else {
                    let mut ovmf_code_final = OsString::from("if=pflash,format=raw,unit=0,file=");
                    ovmf_code_final.push(&ovmf_code);
                    ovmf_code_final.push(",readonly=on");
                    let mut ovmf_vars_final = OsString::from("if=pflash,format=raw,unit=1,file=");
                    ovmf_vars_final.push(&ovmf_vars);
                    let driver = OsString::from("driver=cfi.pflash01,property=secure,value=on");
                    Ok(("Boot: EFI (x86_64), OVMF: ".to_string() + ovmf_code.to_str().unwrap() + ", Secure Boot: " + (*secure_boot).as_str(), 
                            Some(vec!["-global".into(), driver, "-drive".into(), ovmf_code_final, "-drive".into(), ovmf_vars_final])))
                }
            },
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
    fn into_args(self, vmname: &str, ssh: u16, port_forwards: Option<Vec<(u16, u16)>>, publicdir: Option<&OsString>, guest_os: &GuestOS) -> Result<(Vec<String>, Option<Vec<String>>)> {
        match self {
            Self::None => Ok((vec!["-nic".into(), "none".into()], Some(vec!["Network: Disabled".into()]))),
            Self::Restrict | Self::Nat => {
                let mut msgs = Vec::new();
                let port_forwards = port_forwards.map(|forwards| {
                    let data: (Vec<String>, Vec<String>) = forwards.iter()
                        .map(|(host, guest)| (format!(",hostfwd=tcp::{}-:{},hostfwd=udp::{}-:{}", host, guest, host, guest), format!("{} => {}", host, guest)))
                        .unzip();
                    msgs.push(format!("Port forwards: {}", data.1.join(", ")));
                    data.0.join("")
                });

                let net = {
                    let samba = which("smbd").ok().and_then(|_| publicdir.map(|dir| {
                        msgs.push("smbd on guest: `smb://10.0.2.4/qemu`".into());
                        format!(",smb={}", dir.to_string_lossy())
                    })).unwrap_or_default();
                    let ssh = format!(",hostfwd=tcp::{}-:22", ssh);
                    format!("user,hostname={vmname}{ssh}{}{samba}", port_forwards.unwrap_or_default())
                };

                let device = guest_os.net_device();
                let device_arg = format!("{},netdev=nic", device);
                let netdev = format!("{}{},id=nic", net, if self == Self::Restrict { ",restrict=y" } else { "" });


                msgs.insert(0, format!("Network: {} ({}), SSH (On host): ssh user@localhost -p {ssh}", if self == Self::Restrict { "Restricted" } else { "User" }, device));
                Ok((vec!["-netdev".into(), netdev, "-device".into(), device_arg], Some(msgs)))
            },
            Self::Bridged { bridge, mac_addr } => {
                let network = Networks::new_with_refreshed_list();
                if !network.contains_key(&bridge) {
                    bail!("Network interface {} could not be found.", bridge);
                }
                let mac_addr = mac_addr.and_then(|mac| format!(",mac={}", mac).into()).unwrap_or_default();
                Ok((vec!["-nic".into(), "bridge,br=".to_string() + &bridge + &mac_addr], Some(vec!["Network: Bridged (".to_string() + &bridge + ")"])))
            },
        }
    }
}

impl TryInto<Option<OsString>> for PublicDir {
    type Error = anyhow::Error;
    fn try_into(self) -> Result<Option<OsString>> {
        Ok(match self {
            Self::None => None,
            Self::Default => {
                let public = dirs::public_dir();
                if public != dirs::home_dir() {
                    public.map(|dir| dir.into_os_string())
                } else {
                    None
                }
            },
            Self::Custom(dir) => {
                let path = PathBuf::from(&dir);
                if path.exists() {
                    Some(path.into_os_string())
                } else {
                    bail!("Chosen public directory {} does not exist.", dir)
                }
            },
        })
    }
}

fn tpm_args(vm_dir: &Path, vm_name: &str) -> Result<([OsString; 6], String)> {
    let swtpm = which("swtpm").map_err(|_| anyhow!("swtpm must be installed for TPM support."))?;

    let sh_file = vm_dir.join(vm_name.to_string() + ".sh");
    let mut sh_file = OpenOptions::new().append(true).open(sh_file)?;
    let log_file = vm_dir.join(vm_name.to_string() + ".log");
    let log_file = OpenOptions::new().append(true).create(true).open(log_file)?;
    let tpm_socket = vm_dir.join(vm_name.to_string() + ".swtpm-sock");

    let mut ctrl = OsString::from("type=unixio,path=");
    ctrl.push(&tpm_socket);
    let mut tpmstate = OsString::from("dir=");
    tpmstate.push(vm_dir);

    let tpm_args: [OsString; 7] = ["socket".into(),
        "--ctrl".into(), ctrl,
        "--terminate".into(),
        "--tpmstate".into(), tpmstate,
        "--tpm2".into(),
    ];

    let _ = writeln!(sh_file, "{} \\\n{}", swtpm.to_str().unwrap(), tpm_args.iter().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" \\\n    "));
    let pid = Command::new(swtpm).args(tpm_args).stderr(log_file).spawn().map_err(|_| anyhow!("Failed to start swtpm. Please check the log file in your VM directory for more information."))?.id();
    let tpm_print = format!("TPM: {} (pid: {pid})", tpm_socket.to_string_lossy());

    let mut socket = OsString::from("socket,id=chrtpm,path=");
    socket.push(tpm_socket);
    
    Ok((["-chardev".into(), socket,
        "-tpmdev".into(), "emulator,id=tpm0,chardev=chrtpm".into(),
        "-device".into(), "tpm-tis,tpmdev=tpm0".into()], tpm_print))
}


fn cpucores_ram(cores: usize, threads: bool, system_info: &System, ram: u64, guest_os: &GuestOS) -> Result<(Vec<String>, Option<Vec<String>>)> {
    if ram < 4 * (1024 * 1024 * 1024) {
        if let GuestOS::MacOS(_) | GuestOS::Windows | GuestOS::WindowsServer = guest_os {
            bail!("{} guests require at least 4GB of RAM.", guest_os);
        }
    }

    let free_ram = system_info.available_memory() as f64 / BYTES_PER_GB as f64;
    let total_ram = system_info.total_memory() as f64 / BYTES_PER_GB as f64;
    let mut cpus = system_info.cpus().iter()
        .map(|cpu: &Cpu| cpu.brand())
        .collect::<Vec<&str>>();
    // CPUs should already be in a sorted order. remove duplicates.
    cpus.dedup();
    let mut args: Vec<String> = vec!["-smp".into()];

    let sockets = cpus.len();
    let socket_text = match sockets {
        1 => "".to_string(),
        _ => sockets.to_string() + "sockets, "
    };

    let core_text = if cores > 1 && threads {
        args.push(format!("cores={},threads=2,sockets={}", cores / 2, sockets));
        format!("{} core{} and {} threads", cores / 2, if cores > 2 { "s" } else { "" }, cores)
    } else {
        args.push(format!("cores={},threads=1,sockets={}", cores, sockets));
        format!("{} core{}", cores, if cores > 1 { "s" } else { "" })
    };
    args.push("-m".into());
    args.push(ram.to_string() + "b");

    match guest_os {
        GuestOS::MacOS(release) if !matches!(release, MacOSRelease::HighSierra | MacOSRelease::Mojave | MacOSRelease::Catalina) => (),
        _ => {
            args.push("-device".into());
            args.push("virtio-balloon".into());
        },
    };

    Ok((args, Some(vec![format!("Using {}{}, {} GiB of RAM ({:.2} / {:.2} GiB available).", socket_text, core_text, ram as f64 / BYTES_PER_GB as f64, free_ram, total_ram)])))
}

impl USBController {
    fn to_args(&self, guest_os: &GuestOS, smartcard: Option<std::process::Child>, usb_devices: Option<Vec<String>>) -> Result<(Vec<String>, Option<Vec<String>>)> {
        let passthrough_controller = match guest_os {
            GuestOS::MacOS(release) if release < &MacOSRelease::BigSur => "usb-ehci",
            _ => "qemu-xhci",
        };
        let mut args = vec!["-device".into(), "virtio-rng-pci,rng=rng0".into(),
            "-object".into(), "rng-random,id=rng0,filename=/dev/urandom".into(),
            "-device".into(), passthrough_controller.to_string() + ",id=spicepass",
            "-chardev".into(), "spicevmc,id=usbredirchardev1,name=usbredir".into(),
            "-device".into(), "usb-redir,chardev=usbredirchardev1,id=usbredirdev1".into(),
            "-chardev".into(), "spicevmc,id=usbredirchardev2,name=usbredir".into(),
            "-device".into(), "usb-redir,chardev=usbredirchardev2,id=usbredirdev2".into(),
            "-chardev".into(), "spicevmc,id=usbredirchardev3,name=usbredir".into(),
            "-device".into(), "usb-redir,chardev=usbredirchardev3,id=usbredirdev3".into(),
            "-device".into(), "pci-ohci,id=smartpass".into(),
            "-device".into(), "usb-ccid".into(),
        ];
        match self {
            Self::Ehci => args.extend(["-device".into(), "usb-ehci,id=input".into()]),
            Self::Xhci => args.extend(["-device".into(), "qemu-xhci,id=input".into()]),
            _ => ()
        }
        if let Some(child) = smartcard {
            let smartcard = child.wait_with_output()?;
            if std::str::from_utf8(&smartcard.stderr)?.contains("smartcard") {
                args.extend(["-chardev".into(), "spicevmc,id=ccid,name=smartcard".into(),
                    "-device".into(), "ccid-card-passthru,chardev=ccid".into()]);
            } else {
                log::warn!("QEMU was not compiled with support for smartcard devices.");
            }
        };
        if let Some(mut devices) = usb_devices {
            args.extend(["-device".into(), passthrough_controller.to_string() + ",id=hostpass"]);
            args.append(&mut devices);
        }
        Ok((args, None))
    }
}

impl Keyboard {
    fn to_args(&self) -> (Vec<OsString>, Option<Vec<String>>) {
        (match self {
            Self::Usb => vec!["-device".into(), "usb-kbd,bus=input.0".into()],
            Self::Virtio => vec!["-device".into(), "virtio-keyboard".into()],
            Self::PS2 => vec![]
        }, None)
    }
}

impl Mouse {
    fn to_args(&self) -> (Vec<OsString>, Option<Vec<String>>) {
        (match self {
            Self::Usb => vec!["-device".into(), "usb-mouse,bus=input.0".into()],
            Self::Tablet => vec!["-device".into(), "usb-tablet,bus=input.0".into()],
            Self::Virtio => vec!["-device".into(), "virtio-mouse".into()],
            Self::PS2 => vec![],
        }, None)
    }
}

fn publicdir_args(publicdir: &OsString, guest_os: &GuestOS) -> Result<(Vec<OsString>, Option<Vec<String>>)> {
    let mut print_args: Vec<String> = Vec::new();
    let mut args: Vec<OsString> = Vec::new();
    let home_dir = dirs::home_dir().ok_or(anyhow!("Could not find home directory"))?;
    let username = home_dir.file_name().ok_or(anyhow!("Could not find username"))?;
    
    if let GuestOS::MacOS(_) = guest_os {
        print_args.push("WebDAV - On guest: build spice-webdavd (https://gitlab.gnome.org/GNOME/phodav/-/merge_requests/24)\n    Then: Finder -> Connect to Server -> http://localhost:9843/".into());
    } else {
        print_args.push("WebDAV - On guest: dav://localhost:9843/".into());
    }

    match guest_os {
        GuestOS::MacOS(_) => {
            print_args.push("9P - On guest: `sudo mount_9p Public-".to_string() + &username.to_string_lossy() + " ~/Public`");
            if PathBuf::from(publicdir).metadata()?.permissions().readonly() {
                print_args.push("9P - On host - Required for macOS integration: `sudo chmod -r 777 ".to_string() + &publicdir.to_string_lossy() + "`");
            }
        },
        GuestOS::Linux | GuestOS::LinuxOld => print_args.push("9P - On guest: `sudo mount -t 9p -o trans=virtio,version=9p2000.L,msize=104857600 Public-".to_string() + &username.to_string_lossy() + " ~/Public`"),
        _ => (),
    }
    if !matches!(guest_os, GuestOS::Windows | GuestOS::WindowsServer) {
        let mut fs = OsString::from("local,id=fsdev0,path=");
        fs.push(publicdir);
        fs.push(",security_model=mapped-xattr");
        let mut device = OsString::from("virtio-9p-pci,fsdev=fsdev0,mount_tag=Public-");
        device.push(username);
        args.extend(["-fsdev".into(), fs, "-device".into(), device]);
    }

    Ok((args, Some(print_args)))
}

fn basic_args(vm_name: &str, vm_dir: &Path, guest_os: &GuestOS, arch: &Arch) -> Vec<OsString> {
    let mut name = OsString::from(vm_name);
    name.push(",process=");
    name.push(vm_name);
    let mut pid = vm_dir.join(vm_name).into_os_string();
    pid.push(".pid");

    let machine = arch.machine_type(guest_os);

    let mut args = vec!["-name".into(), name, "-pidfile".into(), pid, "-machine".into(), machine];
    if arch.matches_host() {
        args.push("--enable-kvm".into());
    }
    if let Some(mut tweaks) = guest_os.guest_tweaks() {
        args.append(&mut tweaks);
    }
    args
}
