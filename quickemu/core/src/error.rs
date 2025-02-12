use std::fmt;

use size::Size;

use crate::{data::GuestOS, fl};

#[derive(derive_more::From, Debug)]
pub enum ConfigError {
    Read(std::io::Error),
    Parse(toml::de::Error),
}

impl std::error::Error for ConfigError {}
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Read(err) => fl!("read-config-error", err = err.to_string()),
            Self::Parse(err) => fl!("parse-config-error", err = err.to_string()),
        };
        f.write_str(&text)
    }
}

#[derive(derive_more::From, Debug, Clone)]
pub enum Error {
    Instructions(&'static str),
    UnavailablePort(u16),
    InsufficientRam(Size, GuestOS),
    ConflictingSoundUsb,
    #[cfg(not(feature = "inbuilt_commands"))]
    #[from]
    Which(#[from] which::Error),
    Command(&'static str, String),
    LegacyBoot,
    Riscv64Bootloader,
    Ovmf,
    CopyOvmfVars(String),
    UnsupportedBootCombination,
    ViewerNotFound(&'static str),
    QemuNotFound(&'static str),
    DiskCreationFailed(String),
    DiskInUse(String),
    DeserializeQemuImgInfo(String),
    MacBootloader,
    NonexistentImage(String),
    MonitorCommand(String),
}

impl std::error::Error for Error {}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::Instructions(missing_instruction) => {
                let missing_instruction = *missing_instruction;
                fl!("macos-cpu-instructions", instruction = missing_instruction)
            }
            Self::UnavailablePort(port) => fl!("unavailable-port", port = port),
            Self::InsufficientRam(ram, guest) => fl!("insufficient-ram", ram = ram.to_string(), guest = guest.to_string()),
            Self::ConflictingSoundUsb => fl!("sound-usb-conflict"),
            #[cfg(not(feature = "inbuilt_commands"))]
            Self::Which(err) => fl!("which-binary", err = err.to_string()),
            Self::Command(bin, err) => {
                let bin = *bin;
                fl!("failed-launch", bin = bin, err = err)
            }
            Self::LegacyBoot => fl!("non-x86-bios"),
            Self::Riscv64Bootloader => fl!("riscv64-boot"),
            Self::Ovmf => fl!("efi-firmware"),
            Self::CopyOvmfVars(err) => fl!("failed-ovmf-copy", err = err),
            Self::UnsupportedBootCombination => fl!("unsupported-boot-combination"),
            Self::ViewerNotFound(requested_viewer) => {
                let requested_viewer = *requested_viewer;
                fl!("no-viewer", viewer_bin = requested_viewer)
            }
            Self::QemuNotFound(requested_qemu) => {
                let requested_qemu = *requested_qemu;
                fl!("no-qemu", qemu_bin = requested_qemu)
            }
            Self::DiskCreationFailed(err) => fl!("failed-disk-creation", err = err),
            Self::DiskInUse(disk) => fl!("disk-used", disk = disk),
            Self::DeserializeQemuImgInfo(err) => fl!("failed-qemu-img-deserialization", err = err),
            Self::MacBootloader => fl!("no-mac-bootloader"),
            Self::NonexistentImage(requested_image) => fl!("nonexistent-image", img = requested_image),
            Self::MonitorCommand(err) => fl!("monitor-command-failed", err = err),
        };
        f.write_str(&text)
    }
}

#[derive(derive_more::Error, Debug)]
pub enum MonitorError {
    NoMonitor,
    Write(std::io::Error),
    Read(std::io::Error),
}

impl fmt::Display for MonitorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::NoMonitor => fl!("no-monitor-available"),
            Self::Write(err) => fl!("failed-monitor-write", err = err.to_string()),
            Self::Read(err) => fl!("failed-monitor-read", err = err.to_string()),
        };
        f.write_str(&text)
    }
}

#[derive(Debug, Clone)]
pub enum Warning {
    MacOSCorePow2(usize),
    HwVirt(&'static str),
    #[cfg(target_os = "linux")]
    AudioBackend,
    InsufficientRamConfiguration(Size, GuestOS),
}

impl std::error::Error for Warning {}
impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::MacOSCorePow2(recommended) => fl!("macos-core-power-two", recommended = recommended),
            Self::HwVirt(virt_branding) => {
                let virt_branding = *virt_branding;
                fl!("software-virt-fallback", virt_branding = virt_branding)
            }
            Self::AudioBackend => fl!("audio-backend-unavailable"),
            #[cfg(target_os = "linux")]
            Self::InsufficientRamConfiguration(ram, guest) => fl!(
                "insufficient-ram-configuration",
                ram = ram.to_string(),
                guest = guest.to_string()
            ),
        };
        f.write_str(&text)
    }
}
