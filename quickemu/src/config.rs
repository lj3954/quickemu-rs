use anyhow::{bail, Result};
use clap::ValueEnum;
use serde::{de::Visitor, Deserialize, Serialize};
use std::{fmt, net::SocketAddr, path::PathBuf};

#[derive(Debug)]
pub struct Args {
    pub access: Access,
    pub arch: Arch,
    pub braille: bool,
    pub boot: BootType,
    pub cpu_cores: CpuCores,
    pub disk_images: Vec<DiskImage>,
    pub display: Display,
    pub accelerated: bool,
    pub extra_args: Vec<String>,
    pub image_files: Option<Vec<Image>>,
    pub guest_os: GuestOS,
    pub status_quo: bool,
    pub system: sysinfo::System,
    pub network: Network,
    pub port_forwards: Option<Vec<PortForward>>,
    pub public_dir: PublicDir,
    pub ram: u64,
    pub tpm: bool,
    pub usb_devices: Option<Vec<String>>,
    pub viewer: Option<Viewer>,
    pub ssh_port: u16,
    #[cfg(not(target_os = "macos"))]
    pub spice_port: Option<u16>,
    pub monitor: Monitor,
    pub monitor_cmd: Option<String>,
    pub resolution: Resolution,
    pub screenpct: Option<u32>,
    pub serial: Monitor,
    pub usb_controller: USBController,
    pub keyboard: Keyboard,
    pub keyboard_layout: Option<String>,
    pub mouse: Mouse,
    pub sound_card: SoundCard,
    pub fullscreen: bool,
    pub vm_dir: PathBuf,
    pub vm_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigFile {
    pub guest_os: GuestOS,
    #[serde(default)]
    pub arch: Arch,
    #[serde(default, skip_serializing_if = "is_default")]
    pub boot_type: BootType,
    pub cpu_cores: Option<std::num::NonZeroUsize>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub display: Display,
    pub disk_images: Vec<DiskImage>,
    #[serde(default = "default_accel", skip_serializing_if = "is_true")]
    pub accelerated: bool,
    pub image_files: Option<Vec<Image>>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub network: Network,
    pub port_forwards: Option<Vec<PortForward>>,
    pub public_dir: Option<String>,
    #[serde(deserialize_with = "deserialize_size", default)]
    pub ram: Option<u64>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub tpm: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub keyboard: Keyboard,
    pub keyboard_layout: Option<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub monitor: SerdeMonitor,
    #[serde(default, skip_serializing_if = "is_default")]
    pub serial: SerdeMonitor,
    #[serde(default, skip_serializing_if = "is_default")]
    pub soundcard: SoundCard,
    pub mouse: Option<Mouse>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub resolution: Resolution,
    pub usb_controller: Option<USBController>,
    #[serde(default = "default_spice_port", skip_serializing_if = "is_default_spice")]
    pub spice_port: u16,
    #[serde(default = "default_ssh_port", skip_serializing_if = "is_default_ssh")]
    pub ssh_port: u16,
    pub usb_devices: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub fullscreen: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenpct: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viewer: Option<Viewer>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub status_quo: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub braille: bool,
    #[serde(default, skip_serializing_if = "is_default")]
    pub access: Access,
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            guest_os: Default::default(),
            arch: Default::default(),
            boot_type: Default::default(),
            cpu_cores: None,
            display: Default::default(),
            disk_images: Default::default(),
            accelerated: default_accel(),
            image_files: None,
            network: Default::default(),
            port_forwards: None,
            public_dir: None,
            ram: None,
            tpm: false,
            keyboard: Default::default(),
            keyboard_layout: None,
            monitor: Default::default(),
            serial: Default::default(),
            soundcard: Default::default(),
            mouse: None,
            resolution: Default::default(),
            usb_controller: None,
            spice_port: default_spice_port(),
            ssh_port: default_ssh_port(),
            usb_devices: None,
            extra_args: Vec::new(),
            fullscreen: false,
            screenpct: None,
            viewer: None,
            status_quo: false,
            braille: false,
            access: Default::default(),
        }
    }
}
#[cfg(target_os = "macos")]
fn default_accel() -> bool {
    false
}
#[cfg(not(target_os = "macos"))]
fn default_accel() -> bool {
    true
}
fn default_spice_port() -> u16 {
    5930
}
fn default_ssh_port() -> u16 {
    22220
}
fn is_default<T: Default + PartialEq>(input: &T) -> bool {
    input == &T::default()
}
fn is_default_ssh(input: &u16) -> bool {
    *input == default_ssh_port()
}
fn is_default_spice(input: &u16) -> bool {
    *input == default_spice_port()
}
fn is_true(input: &bool) -> bool {
    *input
}

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum Access {
    Remote,
    #[default]
    Local,
    Address(String),
}

#[allow(non_camel_case_types)]
#[derive(clap::ValueEnum, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum Arch {
    #[default]
    #[clap(rename_all = "verbatim")]
    x86_64,
    aarch64,
    riscv64,
}
impl fmt::Display for Arch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::x86_64 => write!(f, "x86_64"),
            Self::aarch64 => write!(f, "aarch64"),
            Self::riscv64 => write!(f, "riscv64"),
        }
    }
}
impl AsRef<str> for Arch {
    fn as_ref(&self) -> &str {
        match self {
            Self::x86_64 => "x86_64",
            Self::aarch64 => "aarch64",
            Self::riscv64 => "riscv64",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BootType {
    #[serde(rename = "efi", alias = "EFI", alias = "Efi")]
    Efi {
        #[serde(default)]
        secure_boot: bool,
    },
    #[serde(rename = "legacy", alias = "Legacy", alias = "bios")]
    Legacy,
}
impl Default for BootType {
    fn default() -> Self {
        Self::Efi { secure_boot: false }
    }
}

#[derive(Debug)]
pub struct CpuCores {
    pub cores: usize,
    pub smt: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskImage {
    pub path: PathBuf,
    #[serde(deserialize_with = "deserialize_size", default)]
    pub size: Option<u64>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub preallocation: PreAlloc,
    pub format: Option<DiskFormat>,
}

struct SizeUnit;
impl<'de> Visitor<'de> for SizeUnit {
    type Value = Option<u64>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string (ending in a size unit, e.g. M, G, T) or a number (in bytes)")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        crate::config_parse::size_unit(value)
            .map(Some)
            .map_err(serde::de::Error::custom)
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(value.try_into().map_err(serde::de::Error::custom)?))
    }
}
pub fn deserialize_size<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_any(SizeUnit)
}

#[derive(Serialize, Clone, Deserialize, Default, Debug, PartialEq)]
pub enum DiskFormat {
    #[serde(alias = "qcow2")]
    #[default]
    Qcow2,
    #[serde(alias = "raw")]
    Raw,
    #[serde(alias = "qed")]
    Qed,
    #[serde(alias = "qcow")]
    Qcow,
    #[serde(alias = "vdi")]
    Vdi,
    #[serde(alias = "vpc")]
    Vpc,
    #[serde(alias = "vhdx")]
    Vhdx,
}
impl AsRef<str> for DiskFormat {
    fn as_ref(&self) -> &str {
        match self {
            Self::Qcow2 => "qcow2",
            Self::Raw => "raw",
            Self::Qed => "qed",
            Self::Qcow => "qcow",
            Self::Vdi => "vdi",
            Self::Vpc => "vpc",
            Self::Vhdx => "vhdx",
        }
    }
}

#[derive(ValueEnum, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Display {
    #[serde(alias = "none")]
    None,
    #[serde(alias = "sdl", alias = "SDL")]
    Sdl,
    #[serde(alias = "gtk", alias = "GTK")]
    Gtk,
    #[cfg(not(target_os = "macos"))]
    #[serde(alias = "spice")]
    Spice,
    #[cfg(not(target_os = "macos"))]
    #[serde(alias = "spice_app", alias = "spice-app")]
    SpiceApp,
    #[cfg(target_os = "macos")]
    #[serde(alias = "cocoa")]
    Cocoa,
}
impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Sdl => write!(f, "SDL"),
            Self::Gtk => write!(f, "GTK"),
            #[cfg(not(target_os = "macos"))]
            Self::Spice => write!(f, "Spice"),
            #[cfg(not(target_os = "macos"))]
            Self::SpiceApp => write!(f, "Spice App"),
            #[cfg(target_os = "macos")]
            Self::Cocoa => write!(f, "Cocoa"),
        }
    }
}
impl Default for Display {
    #[cfg(target_os = "macos")]
    fn default() -> Self {
        Self::Cocoa
    }
    #[cfg(not(target_os = "macos"))]
    fn default() -> Self {
        Self::Sdl
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GuestOS {
    #[serde(rename = "linux", alias = "Linux")]
    #[default]
    Linux,
    #[serde(rename = "linux_old", alias = "LinuxOld")]
    LinuxOld,
    #[serde(rename = "windows", alias = "Windows")]
    Windows,
    #[serde(rename = "windows_server", alias = "WindowsServer")]
    WindowsServer,
    #[serde(alias = "macOS", rename = "macos")]
    MacOS { release: MacOSRelease },
    #[serde(rename = "freebsd", alias = "FreeBSD")]
    FreeBSD,
    #[serde(rename = "ghostbsd", alias = "GhostBSD")]
    GhostBSD,
    #[serde(rename = "dragonflybsd", alias = "DragonFlyBSD")]
    DragonFlyBSD,
    #[serde(rename = "freedos", alias = "FreeDOS")]
    FreeDOS,
    #[serde(rename = "haiku", alias = "Haiku")]
    Haiku,
    #[serde(rename = "solaris", alias = "Solaris")]
    Solaris,
    #[serde(rename = "kolibrios", alias = "KolibriOS")]
    KolibriOS,
    #[serde(rename = "reactos", alias = "ReactOS")]
    ReactOS,
    #[serde(rename = "batocera", alias = "Batocera")]
    Batocera,
}

impl fmt::Display for GuestOS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuestOS::Linux => write!(f, "Linux"),
            GuestOS::LinuxOld => write!(f, "Linux (Old)"),
            GuestOS::Windows => write!(f, "Windows"),
            GuestOS::WindowsServer => write!(f, "Windows Server"),
            GuestOS::MacOS { .. } => write!(f, "macOS"),
            GuestOS::FreeBSD => write!(f, "FreeBSD"),
            GuestOS::GhostBSD => write!(f, "GhostBSD"),
            GuestOS::DragonFlyBSD => write!(f, "DragonFlyBSD"),
            GuestOS::FreeDOS => write!(f, "FreeDOS"),
            GuestOS::Haiku => write!(f, "Haiku"),
            GuestOS::Solaris => write!(f, "Solaris"),
            GuestOS::KolibriOS => write!(f, "KolibriOS"),
            GuestOS::ReactOS => write!(f, "ReactOS"),
            GuestOS::Batocera => write!(f, "Batocera"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Serialize, Deserialize)]
pub enum MacOSRelease {
    #[serde(alias = "highsierra", alias = "10.13")]
    HighSierra,
    #[serde(alias = "mojave", alias = "10.14")]
    Mojave,
    #[serde(alias = "catalina", alias = "10.15")]
    Catalina,
    #[serde(alias = "bigsur", alias = "11")]
    BigSur,
    #[serde(alias = "monterey", alias = "12")]
    Monterey,
    #[serde(alias = "ventura", alias = "13")]
    Ventura,
    #[serde(alias = "sonoma", alias = "14")]
    Sonoma,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum Network {
    #[serde(alias = "none")]
    None,
    #[serde(alias = "restrict")]
    Restrict,
    #[serde(alias = "bridged")]
    Bridged {
        bridge: String,
        #[serde(alias = "MAC Address", alias = "macaddr")]
        mac_addr: Option<String>,
    },
    #[default]
    #[serde(alias = "nat", alias = "NAT", alias = "user")]
    Nat,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Image {
    #[serde(alias = "iso", alias = "ISO")]
    Iso(PathBuf),
    #[serde(alias = "fixed_iso", alias = "cdrom", alias = "CD-ROM")]
    FixedIso(PathBuf),
    #[serde(alias = "floppy")]
    Floppy(PathBuf),
    #[serde(alias = "img", alias = "IMG")]
    Img(PathBuf),
}
impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Iso(path) => write!(f, "Booting from ISO: {}", path.display()),
            Self::Img(path) => write!(f, "Booting from IMG: {}", path.display()),
            Self::FixedIso(path) => write!(f, "Fixed ISO (CD-ROM): {}", path.display()),
            Self::Floppy(path) => write!(f, "Floppy: {}", path.display()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortForward {
    pub host: u16,
    pub guest: u16,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum PreAlloc {
    #[default]
    Off,
    Metadata,
    Falloc,
    Full,
}
impl PreAlloc {
    pub fn qemu_img_arg(&self, disk_format: &DiskFormat) -> Result<Option<&str>> {
        Ok(match (self, disk_format) {
            (Self::Off, DiskFormat::Qcow2) => Some("lazy_refcounts=on,preallocation=off,nocow=on"),
            (Self::Off, DiskFormat::Raw) => Some("preallocation=off"),
            (Self::Metadata, DiskFormat::Qcow2) => Some("lazy_refcounts=on,preallocation=metadata,nocow=on"),
            (Self::Falloc, DiskFormat::Qcow2) => Some("lazy_refcounts=on,preallocation=falloc,nocow=on"),
            (Self::Falloc, DiskFormat::Raw) => Some("preallocation=falloc"),
            (Self::Full, DiskFormat::Qcow2) => Some("lazy_refcounts=on,preallocation=full,nocow=on"),
            (Self::Full, DiskFormat::Raw) => Some("preallocation=full"),
            (Self::Metadata, DiskFormat::Raw) => bail!("`raw` disk images do not support the metadata preallocation type."),
            _ => bail!(
                "Preallocation is not supported for disk format {}. Only `raw` and `qcow2` images have support for the feature",
                disk_format.as_ref()
            ),
        })
    }
}
impl std::fmt::Display for PreAlloc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Off => write!(f, "Disabled"),
            Self::Metadata => write!(f, "metadata"),
            Self::Falloc => write!(f, "falloc"),
            Self::Full => write!(f, "full"),
        }
    }
}

#[derive(Debug)]
pub enum PublicDir {
    None,
    Default,
    Custom(String),
}

#[derive(Debug)]
pub enum Snapshot {
    Apply(String),
    Create(String),
    Delete(String),
    Info,
}

#[derive(Deserialize, Serialize, ValueEnum, Clone, Debug)]
pub enum Viewer {
    None,
    Spicy,
    Remote,
}
impl Default for Viewer {
    fn default() -> Self {
        Self::Spicy
    }
}

#[derive(Debug)]
pub enum Monitor {
    None,
    Telnet { address: SocketAddr },
    Socket { socketpath: PathBuf },
}
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SerdeMonitor {
    #[serde(skip_serializing_if = "is_socket")]
    pub r#type: String,
    pub telnet_host: Option<String>,
    pub telnet_port: Option<u16>,
}
impl Default for SerdeMonitor {
    fn default() -> Self {
        Self {
            r#type: "socket".to_string(),
            telnet_host: None,
            telnet_port: None,
        }
    }
}
fn is_socket(input: &str) -> bool {
    input == "socket"
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum Resolution {
    #[default]
    Default,
    Display(String),
    Custom {
        width: u32,
        height: u32,
    },
}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum USBController {
    None,
    Ehci,
    Xhci,
}

#[derive(Default, ValueEnum, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum Keyboard {
    #[default]
    Usb,
    Virtio,
    PS2,
}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum Mouse {
    Usb,
    Tablet,
    Virtio,
    PS2,
}

#[derive(Default, ValueEnum, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum SoundCard {
    None,
    #[default]
    IntelHDA,
    AC97,
    ES1370,
    SB16,
}

pub enum ActionType {
    Launch,
    Kill,
    MigrateConfig,
    DeleteDisk,
    DeleteVM,
    Snapshot(Snapshot),
    EditConfig,
}

pub trait BooleanDisplay {
    fn as_str(&self) -> &'static str;
}
impl BooleanDisplay for bool {
    fn as_str(&self) -> &'static str {
        if *self {
            "Enabled"
        } else {
            "Disabled"
        }
    }
}
