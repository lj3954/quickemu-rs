use std::{
    borrow::Cow,
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    arg,
    data::{Arch, BootType, GuestOS, Machine},
    error::Error,
    oarg,
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

const SECURE_BOOT_OVMF: &[(&str, &str)] = &[
    ("OVMF/OVMF_CODE_4M.secboot.fd", "OVMF/OVMF_VARS_4M.fd"),
    ("edk2/ovmf/OVMF_CODE.secboot.fd", "edk2/ovmf/OVMF_VARS.fd"),
    ("OVMF/x64/OVMF_CODE.secboot.fd", "OVMF/x64/OVMF_VARS.fd"),
    ("edk2-ovmf/OVMF_CODE.secboot.fd", "edk2-ovmf/OVMF_VARS.fd"),
    ("qemu/ovmf-x86_64-smm-ms-code.bin", "qemu/ovmf-x86_64-smm-ms-vars.bin"),
    ("qemu/edk2-x86_64-secure-code.fd", "qemu/edk2-x86_64-code.fd"),
    ("edk2-ovmf/x64/OVMF_CODE.secboot.fd", "edk2-ovmf/x64/OVMF_VARS.fd"),
    ("edk2/x64/OVMF_CODE.secboot.4m.fd", "edk2/x64/OVMF_VARS.4m.fd"),
];
const EFI_OVMF: &[(&str, &str)] = &[
    ("OVMF/OVMF_CODE_4M.fd", "OVMF/OVMF_VARS_4M.fd"),
    ("edk2/ovmf/OVMF_CODE.fd", "edk2/ovmf/OVMF_VARS.fd"),
    ("OVMF/OVMF_CODE.fd", "OVMF/OVMF_VARS.fd"),
    ("OVMF/x64/OVMF_CODE.fd", "OVMF/x64/OVMF_VARS.fd"),
    ("edk2-ovmf/OVMF_CODE.fd", "edk2-ovmf/OVMF_VARS.fd"),
    ("qemu/ovmf-x86_64-4m-code.bin", "qemu/ovmf-x86_64-4m-vars.bin"),
    ("qemu/edk2-x86_64-code.fd", "qemu/edk2-x86_64-code.fd"),
    ("edk2-ovmf/x64/OVMF_CODE.fd", "edk2-ovmf/x64/OVMF_VARS.fd"),
    ("edk2/x64/OVMF_CODE.4m.fd", "edk2/x64/OVMF_VARS.4m.fd"),
];
const AARCH64_OVMF: [(&str, &str); 1] = [("AAVMF/AAVMF_CODE.fd", "AAVMF/AAVMF_VARS.fd")];
const RISCV64_UBOOT: [&str; 1] = ["/usr/lib/u-boot/qemu-riscv64_smode/u-boot.bin"];

impl Machine {
    pub(crate) fn boot_args(&self, vm_dir: &Path, guest: GuestOS) -> Result<BootArgs, Error> {
        match (&self.boot, self.arch) {
            (BootType::Efi { secure_boot: false }, Arch::X86_64 { .. }) if matches!(guest, GuestOS::MacOS { .. }) => macos_firmware(vm_dir),
            _ if matches!(guest, GuestOS::MacOS { .. }) => Err(Error::UnsupportedBootCombination),
            (BootType::Legacy, Arch::X86_64 { .. }) => Ok(BootArgs::X86_64Bios),
            (BootType::Legacy, _) => Err(Error::LegacyBoot),
            (BootType::Efi { secure_boot: _ }, Arch::Riscv64 { .. }) => find_riscv64_bios(vm_dir),
            (BootType::Efi { secure_boot }, Arch::X86_64 { .. }) => {
                let ovmf = if *secure_boot { SECURE_BOOT_OVMF } else { EFI_OVMF };
                standard_firmware(vm_dir, *secure_boot, ovmf).map(BootArgs::X86_64Efi)
            }
            (BootType::Efi { secure_boot: false }, Arch::AArch64 { .. }) => standard_firmware(vm_dir, false, AARCH64_OVMF.as_slice()).map(BootArgs::AArch64Efi),
            _ => Err(Error::UnsupportedBootCombination),
        }
    }
}

pub(crate) enum BootArgs {
    X86_64Bios,
    X86_64Efi(Efi),
    AArch64Efi(Efi),
    Riscv64Efi(PathBuf),
}

pub(crate) struct Efi {
    code: PathBuf,
    vars: PathBuf,
    secure_boot: bool,
}

impl EmulatorArgs for BootArgs {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let value = match self {
            Self::X86_64Bios => Cow::Borrowed("Legacy/BIOS"),
            Self::X86_64Efi(Efi { code, secure_boot, .. }) => Cow::Owned(format!(
                "EFI (x86_64), OVMF: {}, Secure Boot: {}",
                code.display(),
                if *secure_boot { "Enabled" } else { "Disabled" }
            )),
            Self::AArch64Efi(Efi { code, .. }) => Cow::Owned(format!("EFI (AArch64), OVMF: {}", code.display())),
            Self::Riscv64Efi(bootloader) => Cow::Owned(format!("EFI (Riscv64), Bootloader: {}", bootloader.display())),
        };
        Some(ArgDisplay { name: Cow::Borrowed("Boot"), value })
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        match self {
            Self::X86_64Bios => vec![],
            Self::X86_64Efi(Efi { code, vars, .. }) => {
                let mut ovmf_code_final = OsString::from("if=pflash,format=raw,unit=0,file=");
                ovmf_code_final.push(code);
                ovmf_code_final.push(",readonly=on");
                let mut ovmf_vars_final = OsString::from("if=pflash,format=raw,unit=1,file=");
                ovmf_vars_final.push(vars);
                vec![
                    arg!("-global"),
                    arg!("driver=cfi.pflash01,property=secure,value=on"),
                    arg!("-drive"),
                    oarg!(ovmf_code_final),
                    arg!("-drive"),
                    oarg!(ovmf_vars_final),
                ]
            }
            Self::AArch64Efi(Efi { code, vars, .. }) => {
                let mut aavmf_code_final = OsString::from("node-name=rom,driver=file,filename=");
                aavmf_code_final.push(code);
                aavmf_code_final.push(",read-only=true");
                let mut aavmf_vars_final = OsString::from("node-name=efivars,driver=file,filename=");
                aavmf_vars_final.push(vars);
                vec![arg!("-blockdev"), oarg!(aavmf_code_final), arg!("-blockdev"), oarg!(aavmf_vars_final)]
            }
            Self::Riscv64Efi(bootloader) => {
                vec![arg!("-kernel"), oarg!(bootloader)]
            }
        }
    }
}

fn find_riscv64_bios(vm_dir: &Path) -> Result<BootArgs, Error> {
    let bios_dirs = [&vm_dir.join("bios"), vm_dir];
    let bios = bios_dirs
        .into_iter()
        .filter_map(|dir| dir.read_dir().ok())
        .flat_map(|dir| dir.flatten().map(|file| file.path()))
        .find(|path| path.extension().is_some_and(|ext| ext == "bin"));

    bios.or_else(|| {
        RISCV64_UBOOT
            .iter()
            .map(Path::new)
            .find(|path| path.exists())
            .map(|path| path.to_path_buf())
    })
    .map(BootArgs::Riscv64Efi)
    .ok_or(Error::Riscv64Bootloader)
}

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

fn standard_firmware(vm_dir: &Path, secure_boot: bool, ovmfs: &[(&str, &str)]) -> Result<Efi, Error> {
    let vm_vars = vm_dir.join("OVMF_VARS.fd");

    let share_dir = qemu_share_dir();
    ovmfs
        .iter()
        .map(|(code, vars)| (share_dir.join(code), share_dir.join(vars)))
        .find(|(code, vars)| code.exists() && vars.exists())
        .map(|(code, vars)| {
            if !vm_vars.exists() || vm_vars.metadata().is_ok_and(|m| m.permissions().readonly()) {
                std::fs::copy(vars, &vm_vars).map_err(|e| Error::CopyOvmfVars(e.to_string()))?;
            }
            let code = code.canonicalize().expect("OVMF Code should be a valid path");
            Ok::<_, Error>((code, vm_vars))
        })
        .transpose()?
        .map(|(code, vars)| Efi { code, vars, secure_boot })
        .ok_or(Error::Ovmf)
}

fn macos_firmware(vm_dir: &Path) -> Result<BootArgs, Error> {
    let code = vm_dir.join("OVMF_CODE.fd");
    if !code.exists() {
        return Err(Error::Ovmf);
    }
    let vars = ["OVMF_VARS-1024x768.fd", "OVMF_VARS-1920x1080.fd"]
        .iter()
        .map(|vars| vm_dir.join(vars))
        .find(|vars| vars.exists())
        .ok_or(Error::Ovmf)?;

    Ok(BootArgs::X86_64Efi(Efi { code, vars, secure_boot: false }))
}
