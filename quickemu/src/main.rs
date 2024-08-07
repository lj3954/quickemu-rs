mod actions;
mod config;
mod config_parse;
mod qemu_args;
mod validate;

use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use config::{ActionType, Args, ConfigFile};
use std::{fs::read_to_string, path::PathBuf};

fn main() {
    let args = CliArgs::parse();
    log::debug!("CLI ARGS: {:?}", args);

    env_logger::Builder::new().filter_level(args.verbose.log_level_filter()).init();

    match args.get_action_type() {
        ActionType::MigrateConfig => actions::migrate_config(args.config_file).unwrap_or_exit(),
        ActionType::Launch => {
            let args: Args = args.try_into().unwrap_or_exit();
            args.launch_qemu().unwrap_or_exit();
        }
        ActionType::DeleteVM => args.delete_vm().unwrap_or_exit(),
        ActionType::DeleteDisk => args.try_into().and_then(|args: Args| args.delete_disk()).unwrap_or_exit(),
        ActionType::Snapshot(snapshot) => snapshot.perform_action(args.config_file).unwrap_or_exit(),
        ActionType::EditConfig => todo!(),
        ActionType::Kill => {
            let args: Args = args.try_into().unwrap_or_exit();
            args.kill().unwrap_or_exit();
        }
    }
}

pub trait UnwrapOrExit<T> {
    fn unwrap_or_exit(self) -> T;
}
impl<T> UnwrapOrExit<T> for Result<T> {
    fn unwrap_or_exit(self) -> T {
        self.unwrap_or_else(|e| {
            log::error!("{}", e);
            std::process::exit(1);
        })
    }
}

impl CliArgs {
    fn get_action_type(&self) -> ActionType {
        if self.kill {
            ActionType::Kill
        } else if self.migrate_config {
            ActionType::MigrateConfig
        } else if self.delete_vm {
            ActionType::DeleteVM
        } else if self.delete_disk {
            ActionType::DeleteDisk
        } else if self.edit_config {
            ActionType::EditConfig
        } else if let Some(snapshot) = &self.snapshot {
            ActionType::Snapshot(snapshot.as_slice().try_into().unwrap_or_exit())
        } else {
            ActionType::Launch
        }
    }
}

fn parse_conf(conf_file: Vec<String>) -> Result<(String, ConfigFile)> {
    let valid_position = conf_file
        .iter()
        .position(|arg| (arg.ends_with(".toml") && PathBuf::from(arg).exists()) || PathBuf::from(arg.to_owned() + ".toml").exists());

    let conf_file = match valid_position {
        Some(position) => {
            if conf_file.len() > 1 {
                conf_file
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| *i != position)
                    .for_each(|(_, arg)| {
                        log::error!("Unrecognized argument: {arg}");
                    });
            }
            let file = conf_file[position].clone();

            match &file[file.len() - 5..] {
                ".toml" => file,
                _ => file + ".toml",
            }
        }
        None => {
            let pkg = env!("CARGO_PKG_NAME");
            match conf_file.into_iter().find_map(|arg| {
                let arg = if arg.ends_with(".conf") { arg } else { arg + ".conf" };
                let conf_path = PathBuf::from(&arg);
                if conf_path.exists() {
                    Some((arg, conf_path))
                } else {
                    None
                }
            }) {
                #[cfg(not(feature = "support_bash_conf"))]
                Some((conf, _)) => {
                    bail!(
                        "{pkg} no longer supports '.conf' configuration files.\nPlease convert your configuration file to the TOML format using `{pkg} --migrate-config {conf} {}`.",
                        conf.replace(".conf", ".toml")
                    )
                }
                #[cfg(feature = "support_bash_conf")]
                Some((arg, conf)) => {
                    let conf_data = actions::read_legacy_conf(&conf)?;
                    log::warn!(
                        "Legacy configuration files may be parsed inaccurately, and do not support all of the features of {pkg}. Consider migrating to TOML with `{pkg} --migrate-config {arg} {}`",
                        arg.replace(".conf", ".toml")
                    );
                    return Ok((arg, conf_data));
                }
                None => bail!("You are required to input a valid configuration file."),
            }
        }
    };

    log::info!("Using configuration file: {}", &conf_file);

    let conf_data = read_to_string(&conf_file).with_context(|| format!("Could not read configuration file {conf_file}"))?;
    let conf: ConfigFile = toml::from_str(&conf_data).context("Failed to parse config file")?;
    Ok((conf_file, conf))
}

impl TryFrom<CliArgs> for Args {
    type Error = anyhow::Error;
    fn try_from(args: CliArgs) -> Result<Self> {
        let (conf_file, mut conf) = parse_conf(args.config_file)?;

        let info = config_parse::create_sysinfo();
        log::debug!("{:?}", info);
        let guest_os = &conf.guest_os;

        let conf_file_path = PathBuf::from(&conf_file);
        let conf_file_path = conf_file_path
            .parent()
            .context("The parent directory of the config file cannot be found")?;
        log::debug!("Config file path: {:?}", conf_file_path);

        log::debug!("{:?} {}", conf_file_path, conf_file);
        let vm_dir = conf_file[..conf_file.len() - 5].parse::<PathBuf>()?;

        let vm_name = vm_dir
            .file_name()
            .unwrap()
            .to_os_string()
            .into_string()
            .map_err(|e| anyhow!("Unable to parse VM name: {:?}", e))?;
        log::debug!("Found VM Dir: {:?}, VM Name: {}", vm_dir, vm_name);

        let monitor_socketpath = vm_dir.join(format!("{vm_name}-monitor.socket")).to_path_buf();
        let serial_socketpath = vm_dir.join(format!("{vm_name}-serial.socket")).to_path_buf();

        #[cfg(not(target_os = "macos"))]
        let spice_port = qemu_args::find_port(args.spice_port.unwrap_or(conf.spice_port), 9);

        config_parse::handle_disk_paths(&mut conf.disk_images, conf_file_path)?;
        log::debug!("{:?}", conf);

        Ok(Self {
            access: config::Access::from((args.access, conf.access)),
            arch: conf.arch,
            braille: args.braille || conf.braille,
            boot: conf.boot_type,
            cpu_cores: conf.cpu_cores.try_into()?,
            disk_images: conf.disk_images,
            display: args.display.unwrap_or(conf.display),
            accelerated: conf.accelerated,
            extra_args: [conf.extra_args, args.extra_args].concat(),
            image_files: conf.image_files,
            status_quo: args.status_quo || conf.status_quo,
            network: conf.network,
            port_forwards: conf.port_forwards,
            public_dir: config::PublicDir::from((conf.public_dir, args.public_dir)),
            ram: conf.ram.unwrap_or(config_parse::default_ram(info.total_memory())),
            tpm: conf.tpm,
            keyboard: args.keyboard.unwrap_or(conf.keyboard),
            keyboard_layout: config_parse::keyboard_layout((conf.keyboard_layout, args.keyboard_layout))?,
            monitor: config::Monitor::try_from((
                conf.monitor,
                args.monitor,
                args.monitor_telnet_host,
                args.monitor_telnet_port,
                4440,
                monitor_socketpath,
            ))?,
            monitor_cmd: args.monitor_cmd,
            mouse: args.mouse.or(conf.mouse).unwrap_or(guest_os.into()),
            resolution: (conf.resolution, args.width, args.height, args.screen).into(),
            screenpct: args.screenpct.or(conf.screenpct),
            serial: config::Monitor::try_from((
                conf.serial,
                args.serial,
                args.serial_telnet_host,
                args.serial_telnet_port,
                6660,
                serial_socketpath,
            ))?,
            usb_controller: args.usb_controller.or(conf.usb_controller).unwrap_or(guest_os.into()),
            sound_card: args.sound_card.unwrap_or(conf.soundcard),
            fullscreen: args.fullscreen || conf.fullscreen,
            #[cfg(not(target_os = "macos"))]
            spice_port,
            ssh_port: args.ssh_port.unwrap_or(conf.ssh_port),
            usb_devices: conf.usb_devices,
            viewer: args.viewer.or(conf.viewer),
            system: info,
            vm_name,
            vm_dir,
            guest_os: conf.guest_os,
        })
    }
}

#[derive(Parser, Debug)]
#[clap(group = clap::ArgGroup::new("action").required(true).multiple(true))]
struct CliArgs {
    #[arg(long, group = "action")]
    vm: bool,
    #[arg(long)]
    access: Option<String>,
    #[arg(long)]
    braille: bool,
    #[arg(long, group = "action", conflicts_with_all = &["delete_vm", "snapshot", "edit_config", "migrate_config", "kill"])]
    delete_disk: bool,
    #[arg(long, group = "action", conflicts_with_all = &["delete_disk", "snapshot", "edit_config", "migrate_config", "kill"])]
    delete_vm: bool,
    #[arg(long)]
    display: Option<config::Display>,
    #[arg(long, group = "action", conflicts_with_all = &["delete_vm", "delete_disk", "snapshot", "migrate_config", "kill"])]
    edit_config: bool,
    #[arg(long, requires = "height")]
    width: Option<u32>,
    #[arg(long, requires = "width")]
    height: Option<u32>,
    #[arg(long, conflicts_with_all = &["width", "height"])]
    screen: Option<String>,
    #[arg(long, value_parser = clap::value_parser!(u32).range(1..=100), conflicts_with_all = &["width", "height"])]
    screenpct: Option<u32>,
    #[arg(long)]
    shortcut: bool,
    #[arg(long, group = "action", num_args = 1..=2, allow_hyphen_values = true, conflicts_with_all = &["delete_vm", "delete_disk", "edit_config", "migrate_config", "kill"])]
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
    #[arg(long, conflicts_with = "screenpct")]
    fullscreen: bool,
    #[arg(long)]
    usb_controller: Option<config::USBController>,
    #[arg(long, num_args = 1.., allow_hyphen_values = true)]
    extra_args: Vec<String>,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::WarnLevel>,
    #[arg(required = true)]
    config_file: Vec<String>,
    #[arg(long, group = "action", conflicts_with_all = &["delete_vm", "delete_disk", "edit_config", "snapshot", "kill"])]
    migrate_config: bool,
    #[arg(long, group = "action", conflicts_with_all = &["delete_vm", "delete_disk", "edit_config", "snapshot", "migrate_config"])]
    kill: bool,
}
