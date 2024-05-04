mod config;
mod validate;
mod config_parse;
mod qemu_args;

use clap::Parser;
use anyhow::Result;
use std::path::PathBuf;
use std::collections::HashMap;
use std::process::Command;
use sysinfo::{System, RefreshKind, CpuRefreshKind, MemoryRefreshKind};
use std::fs::OpenOptions;
use std::io::Write;

fn main() {
    let args = CliArgs::parse();
    log::debug!("CLI ARGS: {:?}", args);

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    if args.vm {
        let args = parse_conf_file(args).unwrap();
        let mut sh = OpenOptions::new().create(true).append(true).open(args.vm_dir.join(args.vm_name.clone() + ".sh")).unwrap();
        let (qemu, qemu_args) = args.into_qemu_args().unwrap();
        write!(sh, "{}", qemu_args.iter().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ")).unwrap();
        Command::new(qemu).args(qemu_args).spawn().unwrap();
    } else {
        let args = parse_conf_file(args).unwrap();
        log::debug!("CONFIG ARGS: {:?}", args);
        let _qemu_args = args.into_qemu_args().unwrap();
    }

    
}

fn parse_conf_file(args: CliArgs) -> Result<config::Args> {
    let valid_position = args.config_file.iter().position(|arg| {
        ( arg.ends_with(".conf") && PathBuf::from(arg).exists() ) || PathBuf::from(arg.to_owned() + ".conf").exists()
    });

    let conf_file = match valid_position {
        Some(position) => {
            if args.config_file.len() > 1 {
                args.config_file.iter().enumerate().filter(|(i, _)| *i != position).for_each(|(_, arg)| {
                    log::error!("Unrecognized argument: {}", arg);
                });
            }
            let file = args.config_file[position].clone();

            match &file[file.len()-5..] {
                ".conf" => file,
                _ => file + ".conf",
            }
        },
        None => anyhow::bail!("You are required to input a valid configuration file."),
    };

    log::info!("Using configuration file: {}", &conf_file);
    let conf = std::fs::read_to_string(&conf_file).map_err(|_| anyhow::anyhow!("Configuration file {} does not exist.", &conf_file))?;
    log::debug!("Configuration file content: {}", conf);

    let mut conf: HashMap<String, String> = conf.lines().filter_map(|line| {
        log::debug!("Parsing line: {}", line);
        if line.starts_with('#') || !line.contains('=') {
            return None;
        }
        let split = line.split_once('=').unwrap();
        Some((split.0.to_string(), split.1.trim_matches('"').to_string()))
    }).collect::<HashMap<String, String>>();

    log::debug!("{:?}", conf);

    let info = System::new_with_specifics(RefreshKind::new().with_memory(MemoryRefreshKind::new().with_ram()).with_cpu(CpuRefreshKind::new()));
    log::debug!("{:?}",info);
    let guest_os = config::GuestOS::try_from((conf.remove("guest_os"), conf.remove("macos_release")))?;

    let conf_file_path = PathBuf::from(&conf_file)
        .canonicalize()?
        .parent()
        .ok_or_else(|| anyhow::anyhow!("The parent directory of the config file cannot be found"))?
        .to_path_buf();
    log::debug!("Config file path: {:?}", conf_file_path);

    let disk_img = conf_file_path.join(conf.remove("disk_img").ok_or_else(|| anyhow::anyhow!("Your configuration file must contain a disk image"))?);

    let vm_dir = disk_img.parent().unwrap().to_path_buf();
    let vm_name = vm_dir.file_name().unwrap().to_os_string().into_string().map_err(|e| anyhow::anyhow!("Unable to parse VM name: {:?}", e))?;

    let monitor_socketpath = vm_dir.join(format!("{vm_name}-monitor.socket")).to_path_buf();
    let serial_socketpath = vm_dir.join(format!("{vm_name}-serial.socket")).to_path_buf();
    
    Ok(config::Args {
        access: config::Access::from(args.access),
        arch: config::Arch::try_from(conf.remove("arch"))?,
        braille: args.braille,
        boot: config::BootType::try_from((conf.remove("boot"), conf.remove("secureboot")))?,
        cpu_cores: config_parse::cpu_cores(conf.remove("cpu_cores"), num_cpus::get(), num_cpus::get_physical())?,
        disk_img,
        disk_size: config_parse::size_unit(conf.remove("disk_size"), None)?,
        display: config::Display::try_from((conf.remove("display"), args.display))?,
        accelerated: config_parse::parse_optional_bool(conf.remove("accelerated"), true)?,
        extra_args: args.extra_args,
        floppy: config_parse::parse_optional_path(conf.remove("floppy"), "floppy")?,
        fullscreen: args.fullscreen,
        image_file: config::Image::try_from((conf_file_path.as_path(), conf.remove("iso"), conf.remove("img")))?,
        snapshot: config_parse::snapshot(args.snapshot)?,
        status_quo: args.status_quo,
        fixed_iso: config_parse::parse_optional_path(conf.remove("fixed_iso"), "fixed ISO")?,
        network: config::Network::try_from((conf.remove("network"), conf.remove("macaddr")))?,
        port_forwards: config_parse::port_forwards(conf.remove("port_forwards"))?,
        prealloc: config::PreAlloc::try_from(conf.remove("preallocation"))?,
        public_dir: config::PublicDir::from((conf.remove("public_dir"), args.public_dir)),
        ram: config_parse::size_unit(conf.remove("ram"), Some(info.total_memory()))?.unwrap(),
        tpm: config_parse::parse_optional_bool(conf.remove("tpm"), false)?,
        keyboard: config::Keyboard::try_from((conf.remove("keyboard"), args.keyboard))?,
        keyboard_layout: config_parse::keyboard_layout((conf.remove("keyboard_layout"), args.keyboard_layout))?,
        monitor: config::Monitor::try_from(([(conf.remove("monitor"), conf.remove("monitor_telnet_host"), Some(conf.remove("monitor_telnet_port").and_then(|port| port.parse::<u16>().ok()).unwrap_or(4440))),
            (args.monitor, args.monitor_telnet_host, args.monitor_telnet_port)], monitor_socketpath))?,
        mouse: config::Mouse::try_from((conf.remove("mouse"), args.mouse, &guest_os))?,
        resolution: config::Resolution::try_from((conf.remove("resolution"), args.screen, args.resolution))?,
        serial: config::Monitor::try_from(([(conf.remove("serial"), conf.remove("serial_telnet_host"), Some(conf.remove("serial_telnet_port").and_then(|port| port.parse::<u16>().ok()).unwrap_or(6660))),
            (args.serial, args.serial_telnet_host, args.serial_telnet_port)], serial_socketpath))?,
        usb_controller: config::USBController::try_from((conf.remove("usb_controller"), args.usb_controller, &guest_os))?,
        sound_card: config::SoundCard::try_from((conf.remove("sound_card"), args.sound_card))?,
        spice_port: config_parse::port((conf.remove("spice_port"), args.spice_port), 5930, 9)?,
        ssh_port: config_parse::port((conf.remove("ssh_port"), args.ssh_port), 22220, 9)?,
        usb_devices: config_parse::usb_devices(conf.remove("usb_devices")),
        viewer: args.viewer,
        system: info,
        vm_name,
        vm_dir,
        guest_os,
    })
}


#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(long)]
    access: Option<String>,
    #[arg(long)]
    braille: bool,
    #[arg(long)]
    delete_disk: bool,
    #[arg(long)]
    delete_vm: bool,
    #[arg(long)]
    display: Option<config::Display>,
    #[arg(long)]
    fullscreen: bool,
    #[arg(long)]
    resolution: Option<String>,
    #[arg(long)]
    screen: Option<String>,
    #[arg(long)]
    screenpct: Option<u8>,
    #[arg(long)]
    shortcut: bool,
    #[arg(long, num_args = 1..=2)]
    snapshot: Option<Vec<String>>,
    #[arg(long)]
    status_quo: bool,
    #[arg(long)]
    viewer: Option<config::Viewer>,
    #[arg(long)]
    ssh_port: Option<u16>,
    #[arg(long)]
    spice_port: Option<u16>,
    #[arg(long)]
    public_dir: Option<String>,
    #[arg(long)]
    monitor: Option<String>,
    #[arg(long)]
    monitor_telnet_host: Option<String>,
    #[arg(long)]
    monitor_telnet_port: Option<u16>,
    #[arg(long)]
    monitor_cmd: Option<String>,
    #[arg(long)]
    serial: Option<String>,
    #[arg(long)]
    serial_telnet_host: Option<String>,
    #[arg(long)]
    serial_telnet_port: Option<u16>,
    #[arg(long)]
    keyboard: Option<config::Keyboard>,
    #[arg(long)]
    keyboard_layout: Option<String>,
    #[arg(long)]
    mouse: Option<config::Mouse>,
    #[arg(long)]
    sound_card: Option<config::SoundCard>,
    #[arg(long)]
    usb_controller: Option<config::USBController>,
    #[arg(long, num_args = 1.., allow_hyphen_values = true)]
    extra_args: Option<Vec<String>>,
    #[arg(long)]
    vm: bool,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::WarnLevel>,
    #[arg(required = true)]
    config_file: Vec<String>,
}
