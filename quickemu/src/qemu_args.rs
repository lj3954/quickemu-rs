mod arch;
mod display;
mod external_command;
mod guest_os;
mod images;

use crate::{config::*, config_parse::BYTES_PER_GB};
use anyhow::{anyhow, bail, ensure, Context, Result};
use std::{
    ffi::OsString,
    fs::{create_dir, read_to_string, File, OpenOptions},
    io::Write,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use sysinfo::{Cpu, Networks, Pid, ProcessRefreshKind, RefreshKind, System};
use which::which;

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");

impl Args {
    pub fn launch_qemu(self) -> Result<()> {
        let qemu_bin = match &self.arch {
            Arch::x86_64 => "qemu-system-x86_64",
            Arch::aarch64 => "qemu-system-aarch64",
            Arch::riscv64 => "qemu-system-riscv64",
        };
        let qemu_bin = which(qemu_bin).map_err(|_| anyhow!("Could not find QEMU binary: {qemu_bin}. Please make sure QEMU is installed on your system."))?;

        if !self.vm_dir.exists() {
            create_dir(&self.vm_dir).context("Could not create VM directory")?;
        }

        let mut sh_file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(self.vm_dir.join(self.vm_name.clone() + ".sh"))
            .unwrap();
        writeln!(sh_file, "#!/usr/bin/env bash").unwrap();

        #[cfg(feature = "get_qemu_ver")]
        let qemu_version = external_command::qemu_version_process(&qemu_bin)?;
        let smartcard = if matches!(self.arch, Arch::x86_64 | Arch::riscv64) && cfg!(feature = "check_smartcard") && !cfg!(target_os = "macos") {
            Some(external_command::smartcard_process(&qemu_bin)?)
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

        let mut qemu_args = match basic_args(&self.vm_name, &self.vm_dir, &self.guest_os, &self.arch) {
            Some(args) => args,
            None => {
                if let Some(cmd) = self.monitor_cmd {
                    return self.monitor.send_command(&cmd);
                } else {
                    bail!("VM is already running. Use `{PKG_NAME} --kill {{conf_file}}` to forcibly stop it.",);
                }
            }
        };

        let mut print_args: Vec<String> = Vec::with_capacity(16);
        qemu_args.reserve(32);

        qemu_args.extend(self.display.audio_arg());
        cpucores_ram(self.cpu_cores, &self.system, self.ram, &self.guest_os)?.add_args(&mut qemu_args, &mut print_args);
        if let Some(arg) = self.guest_os.cpu_argument(&self.arch) {
            qemu_args.extend(["-cpu".into(), arg.into()]);
        }

        #[cfg(not(target_os = "macos"))]
        if matches!(self.display, Display::None | Display::Spice | Display::SpiceApp) {
            self.display
                .spice_args(
                    self.spice_port,
                    self.access,
                    (self.accelerated, self.fullscreen),
                    &self.guest_os,
                    publicdir.as_ref(),
                    &self.vm_name,
                )?
                .add_args(&mut qemu_args, &mut print_args);
        }

        self.network
            .into_args(
                &self.vm_name,
                self.ssh_port,
                self.port_forwards,
                publicdir.as_ref(),
                &self.guest_os,
            )?
            .add_args(&mut qemu_args, &mut print_args);
        self.display
            .display_args(
                &self.guest_os,
                &self.arch,
                self.resolution,
                self.screenpct,
                self.accelerated,
                self.fullscreen,
            )?
            .add_args(&mut qemu_args, &mut print_args);
        self.sound_card.to_args().add_args(&mut qemu_args, &mut print_args);

        if self.tpm {
            let (args, print) = tpm_args(&self.vm_dir, &self.vm_name, &mut sh_file)?;
            qemu_args.extend(args);
            print_args.push(print);
        }

        let (print_boot, boot_args) = self.boot.to_args(&self.vm_dir, &self.guest_os, &self.arch)?;
        print_args.push(print_boot);
        if let Some(mut args) = boot_args {
            qemu_args.append(&mut args);
        }

        images::image_args(
            &self.vm_dir,
            self.image_files,
            self.disk_images,
            &self.guest_os,
            self.status_quo,
        )?
        .add_args(&mut qemu_args, &mut print_args);
        self.usb_controller
            .to_args(&self.guest_os, smartcard, self.usb_devices)?
            .add_args(&mut qemu_args, &mut print_args);
        self.keyboard.to_args().add_args(&mut qemu_args, &mut print_args);

        if let Some(layout) = self.keyboard_layout {
            qemu_args.extend(["-k".into(), layout.into()]);
        }
        if self.braille {
            qemu_args.extend(["-chardev".into(), "braille,id=brltty".into(), "-device".into(), "usb-braille,id=usbbrl,chardev=brltty".into()]);
        }
        if let Some(ref publicdir) = publicdir {
            publicdir_args(publicdir, &self.guest_os)?.add_args(&mut qemu_args, &mut print_args);
        }
        self.mouse.to_args().add_args(&mut qemu_args, &mut print_args);
        self.monitor.to_args("monitor")?.add_args(&mut qemu_args, &mut print_args);
        self.serial.to_args("serial")?.add_args(&mut qemu_args, &mut print_args);

        qemu_args.extend(self.extra_args.into_iter().map(|arg| arg.into()).collect::<Vec<OsString>>());

        log::debug!("QEMU ARGS: {:?}", qemu_args);

        sh_file.set_permissions(PermissionsExt::from_mode(0o755)).unwrap();
        write!(
            sh_file,
            "{} {}",
            qemu_bin.to_string_lossy(),
            qemu_args
                .iter()
                .map(|arg| "\"".to_string() + &arg.to_string_lossy() + "\"")
                .collect::<Vec<_>>()
                .join(" ")
        )
        .unwrap();

        external_command::launch_qemu(&qemu_bin, &qemu_args, &self.display)?;

        #[cfg(feature = "get_qemu_ver")]
        {
            let qemu_version = qemu_version.wait_with_output()?;
            let friendly_ver = std::str::from_utf8(&qemu_version.stdout)?
                .split_whitespace()
                .nth(3)
                .context("Failed to get QEMU version.")?;
            let integer_release: u32 = friendly_ver
                .split('.')
                .next()
                .context("Failed to parse QEMU version.")?
                .parse()?;
            ensure!(
                integer_release >= 7,
                "QEMU version 7.0.0 or higher is required. Found version {friendly_ver}."
            );
            println!("QuickemuRS {PKG_VERSION} using {} {friendly_ver}.", qemu_bin.display(),);
        }
        #[cfg(not(feature = "get_qemu_ver"))]
        println!("QuickemuRS {PKG_VERSION} using {}.", qemu_bin.display());

        print_args.iter().for_each(|arg| println!(" - {}", arg));

        if let Some(cmd) = self.monitor_cmd {
            self.monitor.send_command(&cmd)?;
        }
        #[cfg(not(target_os = "macos"))]
        if self.display == Display::Spice {
            self.viewer
                .unwrap_or_default()
                .start(&self.vm_name, publicdir.as_ref(), self.fullscreen, self.spice_port.unwrap())?;
        }

        Ok(())
    }

    pub fn kill(self) -> Result<()> {
        let pid_path = self.vm_dir.join(self.vm_name + ".pid");
        let pid = read_to_string(&pid_path).map_err(|_| anyhow!("Unable to read PID file. Are you sure the VM is active?"))?;
        let pid = pid.trim().parse::<u32>().map_err(|_| anyhow!("Invalid PID: {pid}"))?;

        std::fs::remove_file(pid_path)?;
        let processes = System::new_with_specifics(RefreshKind::new().with_processes(ProcessRefreshKind::new()));
        if let Some(process) = processes.process(Pid::from_u32(pid)) {
            process.kill();
        } else {
            bail!("Process {} does not exist.", pid);
        }

        Ok(())
    }
}

trait AddToLists {
    fn add_args(self, arg_list: &mut Vec<OsString>, print_list: &mut Vec<String>);
}

impl<T> AddToLists for (Vec<T>, Option<Vec<String>>)
where
    T: Into<OsString>,
{
    fn add_args(self, arg_list: &mut Vec<OsString>, print_list: &mut Vec<String>) {
        arg_list.extend(self.0.into_iter().map(|arg| arg.into()));
        if let Some(msgs) = self.1 {
            print_list.extend(msgs);
        }
    }
}

const SECURE_BOOT_OVMF: [(&str, &str); 7] = [
    ("OVMF/OVMF_CODE_4M.secboot.fd", "OVMF/OVMF_VARS_4M.fd"),
    ("edk2/ovmf/OVMF_CODE.secboot.fd", "edk2/ovmf/OVMF_VARS.fd"),
    ("OVMF/x64/OVMF_CODE.secboot.fd", "OVMF/x64/OVMF_VARS.fd"),
    ("edk2-ovmf/OVMF_CODE.secboot.fd", "edk2-ovmf/OVMF_VARS.fd"),
    ("qemu/ovmf-x86_64-smm-ms-code.bin", "qemu/ovmf-x86_64-smm-ms-vars.bin"),
    ("qemu/edk2-x86_64-secure-code.fd", "qemu/edk2-x86_64-code.fd"),
    ("edk2-ovmf/x64/OVMF_CODE.secboot.fd", "edk2-ovmf/x64/OVMF_VARS.fd"),
];
const EFI_OVMF: [(&str, &str); 8] = [
    ("OVMF/OVMF_CODE_4M.fd", "OVMF/OVMF_VARS_4M.fd"),
    ("edk2/ovmf/OVMF_CODE.fd", "edk2/ovmf/OVMF_VARS.fd"),
    ("OVMF/OVMF_CODE.fd", "OVMF/OVMF_VARS.fd"),
    ("OVMF/x64/OVMF_CODE.fd", "OVMF/x64/OVMF_VARS.fd"),
    ("edk2-ovmf/OVMF_CODE.fd", "edk2-ovmf/OVMF_VARS.fd"),
    ("qemu/ovmf-x86_64-4m-code.bin", "qemu/ovmf-x86_64-4m-vars.bin"),
    ("qemu/edk2-x86_64-code.fd", "qemu/edk2-x86_64-code.fd"),
    ("edk2-ovmf/x64/OVMF_CODE.fd", "edk2-ovmf/x64/OVMF_VARS.fd"),
];
const AARCH64_OVMF: [(&str, &str); 1] = [("AAVMF/AAVMF_CODE.fd", "AAVMF/AAVMF_VARS.fd")];
const RISCV_UBOOT: [&str; 1] = ["/usr/lib/u-boot/qemu-riscv64_smode/u-boot.bin"];

fn qemu_share_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    if let Ok(output) = std::process::Command::new("brew").arg("--prefix").arg("qemu").output() {
        if output.status.success() {
            if let Ok(prefix) = std::str::from_utf8(&output.stdout) {
                log::debug!("Found QEMU prefix: {}", prefix);
                return PathBuf::from(prefix.trim()).join("share");
            }
        }
    }
    PathBuf::from("/usr/share")
}

impl BootType {
    fn to_args(&self, vm_dir: &Path, guest_os: &GuestOS, arch: &Arch) -> Result<(String, Option<Vec<OsString>>)> {
        match (self, arch) {
            (Self::Efi { secure_boot: _ }, Arch::riscv64) => {
                if let Some(bios) = find_riscv_bios(vm_dir)? {
                    Ok((
                        "Boot: EFI (RISC-V), Bootloader: ".to_string() + &bios.to_string_lossy(),
                        Some(vec!["-kernel".into(), bios.into()]),
                    ))
                } else {
                    Ok((
                        "Boot: EFI (RISC-V). \x1b[31mWARNING\x1b[0m: Could not find bootloader in your VM directory. VM may fail to boot. Read more: {PLACEHOLDER}".to_string(),
                        None,
                    ))
                }
            }
            (Self::Legacy, Arch::x86_64) => {
                ensure!(!matches!(guest_os, GuestOS::MacOS { .. }), "macOS guests require EFI boot.");
                Ok(("Boot: Legacy/BIOS".to_string(), None))
            }
            (Self::Efi { secure_boot }, _) => {
                let (ovmf_code, ovmf_vars) = match guest_os {
                    GuestOS::MacOS { .. } => {
                        ensure!(!*secure_boot, "macOS guests do not support Secure Boot.");
                        let efi_code = vm_dir.join("OVMF_CODE.fd");
                        ensure!(efi_code.exists(), "macOS firmware \"OVMF_CODE.fd\" could not be found.");
                        let efi_vars = ["OVMF_VARS-1024x768.fd", "OVMF_VARS-1920x1080.fd"]
                            .iter()
                            .find_map(|vars| {
                                let efi_vars = vm_dir.join(vars);
                                if efi_vars.exists() {
                                    Some(efi_vars)
                                } else {
                                    None
                                }
                            })
                            .context("macOS EFI VARS could not be found.")?;
                        (efi_code, efi_vars)
                    }
                    _ => {
                        let vm_vars = vm_dir.join("OVMF_VARS.fd");
                        let (efi_code, extra_vars) = match (arch, secure_boot) {
                            (Arch::aarch64, _) => find_firmware(&AARCH64_OVMF).context("Firmware for aarch64 could not be found.")?,
                            (_, true) => find_firmware(&SECURE_BOOT_OVMF).context("Secure Boot capable firmware could not be found.")?,
                            _ => find_firmware(&EFI_OVMF).context("EFI firmware could not be found. Please install OVMF firmware.")?,
                        };
                        let efi_code = efi_code.canonicalize()?;
                        if !vm_vars.exists() || vm_vars.metadata()?.permissions().readonly() {
                            std::fs::copy(extra_vars, &vm_vars)?;
                        }
                        (efi_code, vm_vars)
                    }
                };
                if arch == &Arch::aarch64 {
                    let mut aavmf_code_final = OsString::from("node-name=rom,driver=file,filename=");
                    aavmf_code_final.push(&ovmf_code);
                    aavmf_code_final.push(",read-only=true");
                    let mut aavmf_vars_final = OsString::from("node-name=efivars,driver=file,filename=");
                    aavmf_vars_final.push(&ovmf_vars);
                    Ok((
                        "Boot: EFI (aarch64), OVMF: ".to_string() + ovmf_code.to_str().unwrap(),
                        Some(vec!["-blockdev".into(), aavmf_code_final, "-blockdev".into(), aavmf_vars_final]),
                    ))
                } else {
                    let mut ovmf_code_final = OsString::from("if=pflash,format=raw,unit=0,file=");
                    ovmf_code_final.push(&ovmf_code);
                    ovmf_code_final.push(",readonly=on");
                    let mut ovmf_vars_final = OsString::from("if=pflash,format=raw,unit=1,file=");
                    ovmf_vars_final.push(&ovmf_vars);
                    let driver = OsString::from("driver=cfi.pflash01,property=secure,value=on");
                    Ok((
                        "Boot: EFI (x86_64), OVMF: ".to_string() + ovmf_code.to_str().unwrap() + ", Secure Boot: " + (*secure_boot).as_str(),
                        Some(vec![
                            "-global".into(),
                            driver,
                            "-drive".into(),
                            ovmf_code_final,
                            "-drive".into(),
                            ovmf_vars_final,
                        ]),
                    ))
                }
            }
            _ => bail!("The specified combination of architecture and boot type is not currently supported."),
        }
    }
}

fn find_firmware(firmware: &[(&str, &str)]) -> Option<(PathBuf, PathBuf)> {
    let share_dir = qemu_share_dir();
    firmware.iter().find_map(|(code, vars)| {
        let code = share_dir.join(code);
        let vars = share_dir.join(vars);
        if code.exists() && vars.exists() {
            Some((code, vars))
        } else {
            None
        }
    })
}

fn find_riscv_bios(vm_dir: &Path) -> Result<Option<PathBuf>> {
    let system_uboot = || {
        RISCV_UBOOT.iter().find_map(|firmware| {
            let u_boot = PathBuf::from(firmware);
            if u_boot.exists() {
                Some(u_boot)
            } else {
                None
            }
        })
    };
    let bios_dirs = [&vm_dir.join("boot"), vm_dir];
    let mut bios = bios_dirs
        .into_iter()
        .filter_map(|directory| {
            directory
                .read_dir()
                .ok()?
                .filter_map(|file| {
                    let path = file.ok()?.path();
                    if path.extension()? == "bin" {
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect::<Vec<PathBuf>>()
                .into()
        })
        .flatten()
        .collect::<Vec<PathBuf>>();
    match bios.len() {
        0 => Ok(system_uboot()),
        1 => Ok(Some(bios.remove(0))),
        _ => bail!("Could not determine the correct RISC-V bootloader. Please ensure that there are not multiple `.bin` files in your VM directory."),
    }
}

impl Network {
    fn into_args(self, vmname: &str, ssh: u16, port_forwards: Option<Vec<PortForward>>, publicdir: Option<&OsString>, guest_os: &GuestOS) -> Result<(Vec<String>, Option<Vec<String>>)> {
        let (ssh_arg, ssh_msg) = match find_port(ssh, 9) {
            Some(port) => {
                let ssh_arg = format!(",hostfwd=tcp::{port}-:22");
                let ssh_msg = format!(", SSH (On host): ssh {{user}}@localhost -p {port}");
                (ssh_arg, ssh_msg)
            }
            None => Default::default(),
        };
        match self {
            Self::None => Ok((vec!["-nic".into(), "none".into()], Some(vec!["Network: Disabled".into()]))),
            Self::Restrict | Self::Nat => {
                let mut msgs = Vec::new();
                let port_forwards = port_forwards.map(|forwards| {
                    let data: (Vec<String>, Vec<String>) = forwards
                        .iter()
                        .map(|pf| {
                            (
                                format!(",hostfwd=tcp::{}-:{},hostfwd=udp::{}-:{}", pf.host, pf.guest, pf.host, pf.guest),
                                format!("{} => {}", pf.host, pf.guest),
                            )
                        })
                        .unzip();
                    msgs.push(format!("Port forwards: {}", data.1.join(", ")));
                    data.0.join("")
                });

                let net = {
                    let samba = which("smbd")
                        .ok()
                        .and_then(|_| {
                            publicdir.map(|dir| {
                                msgs.push("smbd on guest: `smb://10.0.2.4/qemu`".into());
                                format!(",smb={}", dir.to_string_lossy())
                            })
                        })
                        .unwrap_or_default();
                    format!("user,hostname={vmname}{ssh_arg}{}{samba}", port_forwards.unwrap_or_default())
                };

                let device = guest_os.net_device();
                let device_arg = format!("{},netdev=nic", device);
                let netdev = format!("{}{},id=nic", net, if self == Self::Restrict { ",restrict=y" } else { "" });

                msgs.insert(
                    0,
                    format!(
                        "Network: {} ({device}){ssh_msg}",
                        if self == Self::Restrict { "Restricted" } else { "User" },
                    ),
                );
                Ok((vec!["-netdev".into(), netdev, "-device".into(), device_arg], Some(msgs)))
            }
            Self::Bridged { bridge, mac_addr } => {
                let network = Networks::new_with_refreshed_list();
                ensure!(network.contains_key(&bridge), "Network interface {bridge} could not be found.");
                let mac_addr = mac_addr.and_then(|mac| format!(",mac={}", mac).into()).unwrap_or_default();
                Ok((
                    vec!["-nic".into(), "bridge,br=".to_string() + &bridge + &mac_addr],
                    Some(vec!["Network: Bridged (".to_string() + &bridge + ")"]),
                ))
            }
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
            }
            Self::Custom(dir) => {
                let path = PathBuf::from(&dir);
                ensure!(path.exists(), "Chosen public directory {dir} does not exist.");
                Some(path.into_os_string())
            }
        })
    }
}

fn tpm_args(vm_dir: &Path, vm_name: &str, sh_file: &mut File) -> Result<([OsString; 6], String)> {
    let swtpm = which("swtpm").map_err(|_| anyhow!("swtpm must be installed for TPM support."))?;

    let log_file = vm_dir.join(vm_name.to_string() + ".log");
    let log_file = OpenOptions::new().append(true).create(true).open(log_file)?;
    let tpm_socket = vm_dir.join(vm_name.to_string() + ".swtpm-sock");

    let mut ctrl = OsString::from("type=unixio,path=");
    ctrl.push(&tpm_socket);
    let mut tpmstate = OsString::from("dir=");
    tpmstate.push(vm_dir);

    let tpm_args: [OsString; 7] = ["socket".into(), "--ctrl".into(), ctrl, "--terminate".into(), "--tpmstate".into(), tpmstate, "--tpm2".into()];

    let _ = writeln!(
        sh_file,
        "{} \\\n{}",
        swtpm.to_str().unwrap(),
        tpm_args
            .iter()
            .map(|arg| arg.to_string_lossy())
            .collect::<Vec<_>>()
            .join(" \\\n    ")
    );
    let tpm_pid = external_command::tpm_pid(&swtpm, &tpm_args, log_file)?;
    let tpm_print = format!("TPM: {} (pid: {tpm_pid})", tpm_socket.display());

    let mut socket = OsString::from("socket,id=chrtpm,path=");
    socket.push(tpm_socket);

    Ok((
        [
            "-chardev".into(),
            socket,
            "-tpmdev".into(),
            "emulator,id=tpm0,chardev=chrtpm".into(),
            "-device".into(),
            "tpm-tis,tpmdev=tpm0".into(),
        ],
        tpm_print,
    ))
}

fn cpucores_ram(cpu: CpuCores, system_info: &System, ram: u64, guest_os: &GuestOS) -> Result<(Vec<String>, Option<Vec<String>>)> {
    let mut cores = cpu.cores;
    ensure!(
        !(ram < 4 * BYTES_PER_GB && matches!(guest_os, GuestOS::MacOS { .. } | GuestOS::Windows | GuestOS::WindowsServer)),
        "{guest_os} guests require at least 4GB of RAM.",
    );
    if let GuestOS::MacOS { .. } = guest_os {
        if !cores.is_power_of_two() {
            log::warn!("macOS guests usually will not boot with a core count that is not a power of 2. Rounding down.");
            cores = cores.next_power_of_two() >> 1;
        }
    }

    let free_ram = system_info.available_memory() as f64 / BYTES_PER_GB as f64;
    let total_ram = system_info.total_memory() as f64 / BYTES_PER_GB as f64;
    let mut cpus = system_info.cpus().iter().map(|cpu: &Cpu| cpu.brand()).collect::<Vec<&str>>();
    // CPUs should already be in a sorted order. remove duplicates.
    cpus.dedup();
    let mut args: Vec<String> = vec!["-smp".into()];

    let sockets = cpus.len();
    let socket_text = match sockets {
        1 => "".to_string(),
        _ => sockets.to_string() + "sockets, ",
    };

    let core_text = if cores > 1 && cpu.smt {
        let physical_cores = cores / 2;
        args.push(format!("cores={physical_cores},threads=2,sockets={sockets}"));
        format!("{physical_cores} core{} and {cores} threads", if cores > 2 { "s" } else { "" })
    } else {
        args.push(format!("cores={cores},threads=1,sockets={sockets}"));
        format!("{cores} core{}", if cores > 1 { "s" } else { "" })
    };
    args.push("-m".into());
    args.push(ram.to_string() + "b");

    if !matches!(guest_os, GuestOS::MacOS { release } if release < &MacOSRelease::BigSur) {
        args.push("-device".into());
        args.push("virtio-balloon".into());
    }

    Ok((
        args,
        Some(vec![format!(
            "Using {socket_text}{core_text}, {} GiB of RAM ({free_ram:.2} / {total_ram:.2} GiB available).",
            ram as f64 / BYTES_PER_GB as f64,
        )]),
    ))
}

impl USBController {
    fn to_args(&self, guest_os: &GuestOS, smartcard: Option<std::process::Child>, usb_devices: Option<Vec<String>>) -> Result<(Vec<String>, Option<Vec<String>>)> {
        let passthrough_controller = match guest_os {
            GuestOS::MacOS { release } if release < &MacOSRelease::BigSur => "usb-ehci",
            _ => "qemu-xhci",
        };
        let mut args = vec!["-device".into(), "virtio-rng-pci,rng=rng0".into(), "-object".into(), "rng-random,id=rng0,filename=/dev/urandom".into()];

        #[cfg(not(target_os = "macos"))]
        args.extend([
            "-device".into(),
            passthrough_controller.to_string() + ",id=spicepass",
            "-chardev".into(),
            "spicevmc,id=usbredirchardev1,name=usbredir".into(),
            "-device".into(),
            "usb-redir,chardev=usbredirchardev1,id=usbredirdev1".into(),
            "-chardev".into(),
            "spicevmc,id=usbredirchardev2,name=usbredir".into(),
            "-device".into(),
            "usb-redir,chardev=usbredirchardev2,id=usbredirdev2".into(),
            "-chardev".into(),
            "spicevmc,id=usbredirchardev3,name=usbredir".into(),
            "-device".into(),
            "usb-redir,chardev=usbredirchardev3,id=usbredirdev3".into(),
            "-device".into(),
            "pci-ohci,id=smartpass".into(),
            "-device".into(),
            "usb-ccid".into(),
        ]);

        match self {
            Self::Ehci => args.extend(["-device".into(), "usb-ehci,id=input".into()]),
            Self::Xhci => args.extend(["-device".into(), "qemu-xhci,id=input".into()]),
            _ => (),
        }

        let mut add_smartcard_args = || args.extend(["-chardev".into(), "spicevmc,id=ccid,name=smartcard".into(), "-device".into(), "ccid-card-passthru,chardev=ccid".into()]);
        if let Some(child) = smartcard {
            let smartcard = child.wait_with_output()?;
            if std::str::from_utf8(&smartcard.stdout)?.contains("smartcard") {
                add_smartcard_args();
            } else {
                log::warn!("QEMU was not compiled with support for smartcard devices.");
            }
        } else if !(cfg!(feature = "check_smartcard") || cfg!(target_os = "macos")) {
            add_smartcard_args();
        }

        if let Some(mut devices) = usb_devices {
            args.extend(["-device".into(), passthrough_controller.to_string() + ",id=hostpass"]);
            args.append(&mut devices);
        }
        Ok((args, None))
    }
}

impl Keyboard {
    fn to_args(&self) -> (Vec<OsString>, Option<Vec<String>>) {
        (
            match self {
                Self::Usb => vec!["-device".into(), "usb-kbd,bus=input.0".into()],
                Self::Virtio => vec!["-device".into(), "virtio-keyboard".into()],
                Self::PS2 => vec![],
            },
            None,
        )
    }
}

impl Mouse {
    fn to_args(&self) -> (Vec<OsString>, Option<Vec<String>>) {
        (
            match self {
                Self::Usb => vec!["-device".into(), "usb-mouse,bus=input.0".into()],
                Self::Tablet => vec!["-device".into(), "usb-tablet,bus=input.0".into()],
                Self::Virtio => vec!["-device".into(), "virtio-mouse".into()],
                Self::PS2 => vec![],
            },
            None,
        )
    }
}

fn publicdir_args(publicdir: &OsString, guest_os: &GuestOS) -> Result<(Vec<OsString>, Option<Vec<String>>)> {
    let mut print_args: Vec<String> = Vec::new();
    let mut args: Vec<OsString> = Vec::new();
    let home_dir = dirs::home_dir().context("Could not find home directory")?;
    let username = home_dir.file_name().context("Could not find username")?;

    if let GuestOS::MacOS { .. } = guest_os {
        print_args.push("WebDAV - On guest: build spice-webdavd (https://gitlab.gnome.org/GNOME/phodav/-/merge_requests/24)\n    Then: Finder -> Connect to Server -> http://localhost:9843/".into());
    } else {
        print_args.push("WebDAV - On guest: dav://localhost:9843/".into());
    }

    match guest_os {
        GuestOS::MacOS { .. } => {
            print_args.push("9P - On guest: `sudo mount_9p Public-".to_string() + &username.to_string_lossy() + " ~/Public`");
            if PathBuf::from(publicdir).metadata()?.permissions().readonly() {
                print_args.push("9P - On host - Required for macOS integration: `sudo chmod -r 777 ".to_string() + &publicdir.to_string_lossy() + "`");
            }
        }
        GuestOS::Linux | GuestOS::LinuxOld => {
            print_args.push("9P - On guest: `sudo mount -t 9p -o trans=virtio,version=9p2000.L,msize=104857600 Public-".to_string() + &username.to_string_lossy() + " ~/Public`")
        }
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

fn basic_args(vm_name: &str, vm_dir: &Path, guest_os: &GuestOS, arch: &Arch) -> Option<Vec<OsString>> {
    let pid = vm_dir.join(vm_name.to_owned() + ".pid");
    if pid.exists() {
        return None;
    }

    let rtc = match arch {
        Arch::x86_64 => "base=localtime,clock=host,driftfix=slew",
        _ => "base=localtime,clock=host",
    };
    let machine = arch.machine_type(guest_os);

    let mut args: Vec<OsString> = vec!["-pidfile".into(), pid.into(), "-machine".into(), machine, "-rtc".into(), rtc.into()];

    #[cfg(not(target_os = "macos"))]
    {
        let mut name = OsString::from(vm_name);
        name.push(",process=");
        name.push(vm_name);
        args.extend(["-name".into(), name]);
    }

    if arch.enable_hw_virt() {
        #[cfg(target_os = "linux")]
        args.extend(["-accel".into(), "kvm".into()]);
        #[cfg(target_os = "macos")]
        args.extend(["-accel".into(), "hvf".into()]);
    }
    args.append(&mut guest_os.guest_tweaks(arch));
    Some(args)
}

pub fn find_port(default: u16, offset: u16) -> Option<u16> {
    (default..=default + offset).find(|port| {
        let port = SocketAddrV4::new(Ipv4Addr::LOCALHOST, *port);
        TcpListener::bind(port).is_ok()
    })
}
