use clap::ValueEnum;
use std::path::PathBuf;
use std::fmt;

#[derive(Debug)]
pub struct Args {
    pub access: Access,
    pub arch: Arch,
    pub braille: bool,
    pub boot: BootType,
    pub cpu_cores: (usize, bool),
    pub disk_img: PathBuf,
    pub disk_size: Option<u64>,
    pub display: Display,
    pub accelerated: bool,
    pub extra_args: Option<Vec<String>>,
    pub floppy: Option<PathBuf>,
    pub fullscreen: bool,
    pub image_file: Image,
    pub fixed_iso: Option<PathBuf>,
    pub guest_os: GuestOS,
    pub snapshot: Option<Snapshot>,
    pub status_quo: bool,
    pub network: Network,
    pub port_forwards: Option<Vec<(u16, u16)>>,
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

#[derive(Debug, PartialEq)]
pub enum Access {
    Remote,
    Local,
    Address(String),
}

#[allow(non_camel_case_types)]
#[derive(Debug, PartialEq)]
pub enum Arch {
    x86_64,
    aarch64,
    riscv64,
}
impl Arch {
    pub fn matches_host(&self) -> bool {
        match self {
            Self::x86_64 => cfg!(target_arch = "x86_64"),
            Self::aarch64 => cfg!(target_arch = "aarch64"),
            Self::riscv64 => cfg!(target_arch = "riscv64"),
        }
    }
}

#[derive(Debug)]
pub enum BootType {
    Efi { secure_boot: bool },
    Legacy,
}

#[derive(ValueEnum, Clone, Debug)]
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

#[derive(Debug, PartialEq)]
pub enum GuestOS {
    Linux,
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

#[derive(Debug, PartialEq, PartialOrd)]
pub enum MacOSRelease {
    HighSierra,
    Mojave,
    Catalina,
    BigSur,
    Monterey,
    Ventura,
    Sonoma
}

#[derive(Debug, PartialEq)]
pub enum Network {
    None,
    Restrict,
    Bridged { bridge: String, mac_addr: Option<String> },
    Nat,
}

#[derive(Debug, PartialEq)]
pub enum Image {
    None,
    Iso(PathBuf),
    Img(PathBuf),
}
impl fmt::Display for Image {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Iso(path) => write!(f, "Booting from ISO: {}", path.display()),
            Self::Img(path) => write!(f, "Booting from IMG: {}", path.display()),
        }
    }
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub enum Resolution {
    Default,
    Display(String),
    Custom { width: u32, height: u32 },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum USBController {
    None,
    Ehci,
    Xhci,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Keyboard {
    Usb,
    Virtio,
    PS2,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Mouse {
    Usb,
    Tablet,
    Virtio,
    PS2,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum SoundCard {
    None,
    IntelHDA,
    AC97,
    ES1370,
    SB16,
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
