use clap::ValueEnum;

pub struct Args {
    pub access: Access,
    pub arch: Option<String>,
    pub braille: bool,
    pub boot: BootType,
    pub cpu_cores: u32,
    pub disk_img: String,
    pub disk_size: Option<u32>,
    pub display: Display,
    pub extra_args: Vec<String>,
    pub floppy: Option<String>,
    pub fullscreen: bool,
    pub image_file: String,
    pub second_image_file: Option<String>,
    pub snapshot: Option<Snapshot>,
    pub macos_release: Option<String>,
    pub network: Network,
    pub port_forwards: Vec<String>,
    pub prealloc: bool,
    pub public_dir: PublicDir,
    pub ram: u64,
    pub secure_boot: bool,
    pub tpm: bool,
    pub usb_devices: Vec<String>,
    pub viewer: Option<Viewer>,
    pub ssh_port: u32,
    pub spice_port: u32,
    pub monitor: Monitor,
    pub resolution: Resolution,
    pub serial: Monitor,
    pub usb_controller: USBController,
    pub keyboard: Keyboard,
    pub keyboard_layout: Option<String>,
    pub mouse: Mouse,
    pub sound_card: SoundCard,
}

pub enum Access {
    Remote,
    Local,
    Address(String),
}

pub enum BootType {
    EFI,
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

pub enum Network {
    None,
    Restrict,
    Bridged { mac_addr: Option<String> },
    NAT,
}

pub enum PublicDir {
    None,
    Default,
    Custom(String),
}

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

pub enum Monitor {
    None,
    Telnet { port: u32, host: String },
    Socket { socketpath: String },
}

#[derive(ValueEnum, Clone, Debug)]
pub enum MonitorType {
    None,
    Telnet,
    Socket,
}

pub enum Resolution {
    Default,
    Display(String),
    Custom { width: u32, height: u32 },
}

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
