pub mod config;
pub mod config_parse;
pub mod validate;

#[cfg(feature = "control_launch")]
mod actions;
#[cfg(feature = "control_launch")]
mod qemu_args;

#[cfg(feature = "control_launch")]
pub mod direct_control {
    use crate::{
        config::{self, ActionType, Args, ConfigFile},
        config_parse,
    };
    use anyhow::{bail, Result};
    use std::path::{Path, PathBuf};

    pub fn with_toml_config(config: impl AsRef<Path>, action: ActionType, vm_name: Option<String>, vm_dir: Option<PathBuf>) -> Result<()> {
        let path = config.as_ref();
        let conf_file_path = path.parent().context("No parent directory")?;

        let vm_dir = vm_dir.unwrap_or_else(|| conf_file_path.join(path.file_stem().unwrap()));
        let vm_name = vm_name.unwrap_or_else(|| "default".to_string());

        let config_text = std::fs::read_to_string(path)?;
        let mut conf: ConfigFile = toml::from_str(&config_text)?;

        config_parse::handle_disk_paths(&mut conf.disk_images, conf_file_path)?;

        handle_action(conf, vm_dir, vm_name, action)?;
        Ok(())
    }

    pub fn handle_action(config: ConfigFile, vm_dir: PathBuf, vm_name: String, action: ActionType) -> Result<()> {
        match action {
            ActionType::Launch => {
                let args = generate_args(config, vm_dir, vm_name)?;
                args.launch_qemu()?
            }
            ActionType::Kill => {
                let args = generate_args(config, vm_dir, vm_name)?;
                args.kill()?
            }
            ActionType::Snapshot(snapshot) => {
                snapshot.perform_on_config(config)?;
            }
            _ => bail!("Unimplemented action"),
        }
        Ok(())
    }

    fn generate_args(conf: ConfigFile, vm_dir: PathBuf, vm_name: String) -> Result<Args> {
        let guest_os = &conf.guest_os;
        let system = config_parse::create_sysinfo();

        #[cfg(not(target_os = "macos"))]
        let spice_port = crate::qemu_args::find_port(conf.spice_port, 9);

        let monitor_socketpath = vm_dir.join(format!("{vm_name}-monitor.socket")).to_path_buf();
        let serial_socketpath = vm_dir.join(format!("{vm_name}-serial.socket")).to_path_buf();

        Ok(Args {
            access: conf.access,
            accelerated: conf.accelerated,
            arch: conf.arch,
            braille: conf.braille,
            boot: conf.boot_type,
            cpu_cores: config_parse::cpu_cores(conf.cpu_cores, num_cpus::get(), num_cpus::get_physical())?,
            disk_images: conf.disk_images,
            display: conf.display,
            extra_args: conf.extra_args,
            image_files: conf.image_files,
            status_quo: conf.status_quo,
            network: conf.network,
            port_forwards: conf.port_forwards,
            public_dir: config::PublicDir::from((conf.public_dir, None)),
            ram: conf.ram.unwrap_or(config_parse::default_ram(system.total_memory())),
            tpm: conf.tpm,
            keyboard: conf.keyboard,
            keyboard_layout: config_parse::keyboard_layout((conf.keyboard_layout, None))?,
            monitor: config::Monitor::try_from((conf.monitor, None, None, None, 4440, monitor_socketpath))?,
            mouse: conf.mouse.unwrap_or(guest_os.into()),
            resolution: conf.resolution,
            screenpct: conf.screenpct,
            serial: config::Monitor::try_from((conf.serial, None, None, None, 6660, serial_socketpath))?,
            usb_controller: conf.usb_controller.unwrap_or(guest_os.into()),
            sound_card: conf.soundcard,
            fullscreen: conf.fullscreen,
            #[cfg(not(target_os = "macos"))]
            spice_port,
            ssh_port: conf.ssh_port,
            usb_devices: conf.usb_devices,
            viewer: conf.viewer,
            system,
            vm_name,
            vm_dir,
            guest_os: conf.guest_os,
            monitor_cmd: None,
        })
    }
}
