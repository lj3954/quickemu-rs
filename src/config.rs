use clap::ValueEnum;
use std::path::PathBuf;
use std::fmt;
use serde::{Serialize, Deserialize};

#[derive(Debug)]
pub struct Args {
    pub access: Access,
    pub arch: Arch,
    pub braille: bool,
    pub boot: BootType,
    pub cpu_cores: (usize, bool),
    pub disk_images: Vec<(PathBuf, Option<u64>)>,
    pub display: Display,
    pub accelerated: bool,
    pub extra_args: Option<Vec<String>>,
    pub fullscreen: bool,
    pub image_files: Option<Vec<Image>>,
    pub guest_os: GuestOS,
    pub status_quo: bool,
    pub system: sysinfo::System,
    pub network: Network,
    pub port_forwards: Option<Vec<PortForward>>,
    pub prealloc: PreAlloc,
    pub public_dir: PublicDir,
    pub ram: u64,
    pub tpm: bool,
    pub usb_devices: Option<Vec<String>>,
    pub viewer: Option<Viewer>,
    pub ssh_port: u16,
    pub spice_port: u16,
    pub monitor: Monitor,
    pub resolution: Resolution,
    pub serial: Monitor,
    pub usb_controller: USBController,
    pub keyboard: Keyboard,
    pub keyboard_layout: Option<String>,
    pub mouse: Mouse,
    pub sound_card: SoundCard,
    pub vm_dir: PathBuf,
    pub vm_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigFile {
    pub guest_os: GuestOS,
    #[serde(default = "default_arch")]
    pub arch: Arch,
    #[serde(default = "default_boottype")]
    pub boot_type: BootType,
    pub cpu_cores: Option<std::num::NonZeroUsize>,
    #[serde(default = "default_display")]
    pub display: Display,
    pub disk_images: Vec<DiskImage>,
    #[serde(default = "true_bool")]
    pub accelerated: bool,
    pub image_files: Option<Vec<Image>>,
    #[serde(default = "default_network")]
    pub network: Network,
    pub port_forwards: Option<Vec<PortForward>>,
    #[serde(default = "default_prealloc")]
    pub preallocation: PreAlloc,
    pub public_dir: Option<String>,
    pub ram: Option<String>,
    #[serde(default)]
    pub tpm: bool,
    #[serde(default = "default_keyboard")]
    pub keyboard: Keyboard,
    pub keyboard_layout: Option<String>,
    #[serde(default = "default_monitor")]
    pub monitor: SerdeMonitor,
    #[serde(default = "default_monitor")]
    pub serial: SerdeMonitor,
    #[serde(default = "default_soundcard")]
    pub soundcard: SoundCard,
    pub mouse: Option<Mouse>,
    #[serde(default = "default_resolution")]
    pub resolution: Resolution,
    pub usb_controller: Option<USBController>,
    #[serde(default = "default_spice_port")]
    pub spice_port: u16,
    #[serde(default = "default_ssh_port")]
    pub ssh_port: u16,
    pub usb_devices: Option<Vec<String>>,
}
fn true_bool() -> bool { true }
fn default_arch() -> Arch { Arch::x86_64 }
fn default_boottype() -> BootType { BootType::Efi { secure_boot: false } }
fn default_display() -> Display { Display::Sdl }
fn default_spice_port() -> u16 { 5930 }
fn default_ssh_port() -> u16 { 22220 }
fn default_resolution() -> Resolution { Resolution::Default }
fn default_prealloc() -> PreAlloc { PreAlloc::Off }
fn default_keyboard() -> Keyboard { Keyboard::Usb }
fn default_soundcard() -> SoundCard { SoundCard::IntelHDA }
fn default_network() -> Network { Network::Nat }
fn default_monitor() -> SerdeMonitor { SerdeMonitor { monitor_type: "socket".to_string(), telnet_host: None, telnet_port: None } }

#[derive(Debug, PartialEq)]
pub enum Access {
    Remote,
    Local,
    Address(String),
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Arch {
    x86_64,
    aarch64,
    riscv64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BootType {
    Efi { secure_boot: bool },
    Legacy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskImage {
    pub path: PathBuf,
    pub size: Option<String>,
}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum Display {
    None,
    Sdl,
    Gtk,
    Spice,
    SpiceApp,
}
impl fmt::Display for Display {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Sdl => write!(f, "SDL"),
            Self::Gtk => write!(f, "GTK"),
            Self::Spice => write!(f, "Spice"),
            Self::SpiceApp => write!(f, "Spice App"),
        }
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum GuestOS {
    Linux,
    LinuxOld,
    Windows,
    WindowsServer,
    MacOS(MacOSRelease),
    FreeBSD,
    GhostBSD,
    FreeDOS,
    Haiku,
    Solaris,
    KolibriOS,
    ReactOS,
    Batocera,
}

impl fmt::Display for GuestOS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuestOS::Linux => write!(f, "Linux"),
            GuestOS::LinuxOld => write!(f, "Linux (Old)"),
            GuestOS::Windows => write!(f, "Windows"),
            GuestOS::WindowsServer => write!(f, "Windows Server"),
            GuestOS::MacOS(_) => write!(f, "macOS"),
            GuestOS::FreeBSD => write!(f, "FreeBSD"),
            GuestOS::GhostBSD => write!(f, "GhostBSD"),
            GuestOS::FreeDOS => write!(f, "FreeDOS"),
            GuestOS::Haiku => write!(f, "Haiku"),
            GuestOS::Solaris => write!(f, "Solaris"),
            GuestOS::KolibriOS => write!(f, "KolibriOS"),
            GuestOS::ReactOS => write!(f, "ReactOS"),
            GuestOS::Batocera => write!(f, "Batocera"),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum MacOSRelease {
    HighSierra,
    Mojave,
    Catalina,
    BigSur,
    Monterey,
    Ventura,
    Sonoma
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Network {
    None,
    Restrict,
    Bridged { bridge: String, mac_addr: Option<String> },
    Nat,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Image {
    Iso(PathBuf),
    FixedIso(PathBuf),
    Floppy(PathBuf),
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
    host: u16,
    guest: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum PreAlloc {
    Off,
    Metadata,
    Falloc,
    Full,
}
impl PreAlloc {
    pub fn qemu_arg(&self) -> &'static str {
        match self {
            Self::Off => "lazy_refcounts=on,preallocation=off",
            Self::Metadata => "lazy_refcounts=on,preallocation=metadata",
            Self::Falloc => "lazy_refcounts=on,preallocation=falloc",
            Self::Full => "lazy_refcounts=on,preallocation=full",
        }
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

#[derive(ValueEnum, Clone, Debug)]
pub enum Viewer {
    None,
    Spicy,
    Remote,
}

#[derive(Debug)]
pub enum Monitor {
    None,
    Telnet { port: u16, host: String },
    Socket { socketpath: PathBuf },
}
#[derive(Serialize, Deserialize)]
pub struct SerdeMonitor {
    pub monitor_type: String,
    pub telnet_host: Option<String>,
    pub telnet_port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Resolution {
    Default,
    Display(String),
    Custom { width: u32, height: u32 },
}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum USBController {
    None,
    Ehci,
    Xhci,
}

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum Keyboard {
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

#[derive(ValueEnum, Clone, Debug, Serialize, Deserialize)]
pub enum SoundCard {
    None,
    IntelHDA,
    AC97,
    ES1370,
    SB16,
}

pub enum ActionType {
    Launch,
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
