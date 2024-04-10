mod config;
mod validate;

use clap::Parser;
use anyhow::Result;

fn main() {
    let args = CliArgs::parse();
    println!("{:?}", args);

    env_logger::Builder::new()
        .filter_level(args.verbose.log_level_filter())
        .init();

    let args = parse_conf_file(args).unwrap();
}

fn parse_conf_file(args: CliArgs) -> Result<config::Args> {
    let conf_file = match args.config_file.iter().position(|arg| arg.ends_with(".conf")) {
        Some(position) => {
            if args.config_file.len() > 1 {
                args.config_file.iter().enumerate().filter(|(i, _)| *i != position).for_each(|(_, arg)| {
                    log::error!("Unrecognized argument: {}", arg);
                });
            }
            args.config_file.get(position).unwrap()
        },
        None => {
            log::error!("You are required to input a configuration file.");
            std::process::exit(1);
        }
    };

    log::info!("Using configuration file: {}", conf_file);
    let conf = std::fs::read_to_string(conf_file).map_err(|_| anyhow::anyhow!("Configuration file {} does not exist.", conf_file))?;
    log::debug!("Configuration file content: {}", conf);


    


    todo!()
}


#[derive(Parser, Debug)]
struct CliArgs {
    #[arg(long)]
    access: bool,
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
    ssh_port: Option<u32>,
    #[arg(long)]
    spice_port: Option<u32>,
    #[arg(long)]
    public_dir: Option<String>,
    #[arg(long)]
    monitor: Option<config::MonitorType>,
    #[arg(long)]
    monitor_telnet_host: Option<String>,
    #[arg(long)]
    monitor_telnet_port: Option<u32>,
    #[arg(long)]
    monitor_cmd: Option<String>,
    #[arg(long)]
    serial: Option<config::MonitorType>,
    #[arg(long)]
    serial_telnet_host: Option<String>,
    #[arg(long)]
    serial_telnet_port: Option<u32>,
    #[arg(long)]
    keyboard: Option<config::Keyboard>,
    #[arg(long)]
    keyboard_layout: Option<String>,
    #[arg(long)]
    mouse: Option<config::Mouse>,
    #[arg(long)]
    sound_card: Option<config::SoundCard>,
    #[arg(long)]
    extra_args: Option<Vec<String>>,
    #[arg(long)]
    vm: bool,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
    #[arg(required = true)]
    config_file: Vec<String>,
}
