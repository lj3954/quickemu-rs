use clap::ValueEnum;

#[derive(Debug)]
pub struct Args {
    pub access: Access,
    pub arch: Arch,
    pub braille: bool,
    pub boot: BootType,
    pub cpu_cores: (usize, bool),
    pub disk_img: std::path::PathBuf,
    pub disk_size: Option<u64>,
    pub display: Display,
    pub extra_args: Option<Vec<String>>,
    pub floppy: Option<String>,
    pub fullscreen: bool,
    pub image_file: Image,
    pub fixed_iso: Option<String>,
    pub guest_os: GuestOS,
    pub snapshot: Option<Snapshot>,
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
    pub vm_dir: std::path::PathBuf,
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

#[derive(Debug)]
pub enum BootType {
    EFI { secure_boot: bool },
    Legacy,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Display {
    None,
    SDL,
    GTK,
    Spice,
    SpiceApp,
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

impl std::fmt::Display for GuestOS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
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

#[derive(Debug, PartialEq)]
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
    NAT,
}

#[derive(Debug)]
pub enum Image {
    None,
    ISO(String),
    IMG(String),
}

#[derive(Debug)]
pub enum PreAlloc {
    Off,
    Metadata,
    Falloc,
    Full,
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
    RemoteViewer,
}

#[derive(Debug)]
pub enum Monitor {
    None,
    Telnet { port: u16, host: String },
    Socket { socketpath: std::path::PathBuf },
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
    EHCI,
    XHCI,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Keyboard {
    USB,
    Virtio,
    PS2,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Mouse {
    USB,
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
