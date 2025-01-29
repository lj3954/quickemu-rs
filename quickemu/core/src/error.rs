use size::Size;
use thiserror::Error;

use crate::data::GuestOS;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not read config file: {0}")]
    Read(#[from] std::io::Error),
    #[error("Could not parse config file: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("CPU does not support the necessary instruction for this macOS release: {0}.")]
    Instructions(&'static str),
    #[error("Requested port {0} is unavailable.")]
    UnavailablePort(u16),
    #[error("System RAM {0} is insufficient for {1} VMs.")]
    InsufficientRam(Size, GuestOS),
    #[error("USB Audio requires the XHCI USB controller.")]
    ConflictingSoundUsb,
    #[cfg(not(feature = "inbuilt_commands"))]
    #[error("Could not find binary: {0}")]
    Which(#[from] which::Error),
    #[error("Failed to launch {0}: {1}")]
    Command(&'static str, String),
    #[error("Legacy boot is only supported on x86_64.")]
    LegacyBoot,
    #[error("Could not find riscv64 bootloader")]
    Riscv64Bootloader,
    #[error("Could not find EFI firmware")]
    Ovmf,
    #[error("Could not copy OVMF vars into VM directory: {0}")]
    CopyOvmfVars(String),
    #[error("Specified architecture and boot type are not compatible")]
    UnsupportedBootCombination,
    #[error("Could not find viewer {0}")]
    ViewerNotFound(&'static str),
    #[error("Could not find qemu binary: {0}")]
    QemuNotFound(&'static str),
    #[error("Could not create disk image: {0}")]
    DiskCreationFailed(String),
    #[error("Failed to get write lock on disk {0}. Ensure that it is not already in use.")]
    DiskInUse(String),
    #[error("Could not deserialize qemu-img info: {0}")]
    DeserializeQemuImgInfo(String),
    #[error("Could not find macOS bootloader in VM directory")]
    MacBootloader,
}

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("No monitor is enabled.")]
    NoMonitor,
    #[error("Could not write to the monitor: {0}")]
    Write(std::io::Error),
    #[error("Could not read from the monitor: {0}")]
    Read(std::io::Error),
}

#[derive(Error, Debug, Clone)]
pub enum Warning {
    #[error("macOS guests may not boot with core counts that are not powers of two. Recommended rounding: {0}.")]
    MacOSCorePow2(usize),
    #[error("Hardware virtualization{0} is not enabled on your CPU. Falling back to software virtualization, performance will be degraded")]
    HwVirt(&'static str),
    #[cfg(target_os = "linux")]
    #[error("Sound was requested, but no audio backend could be detected (PipeWire/PulseAudio).")]
    AudioBackend,
    #[error("The specified amount of RAM ({0}) is insufficient for {1}. Performance issues may arise")]
    InsufficientRamConfiguration(Size, GuestOS),
}
