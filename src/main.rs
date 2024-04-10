mod config;
mod validate;

use clap::Parser;

fn main() {
    let args = CliArgs::parse();
    println!("{:?}", args);
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
}
