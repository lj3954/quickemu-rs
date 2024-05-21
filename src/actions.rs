use anyhow::{anyhow, bail, Result};
use crate::config::Snapshot;
use which::which;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::io::Write;
use std::fs::{read_to_string, write, set_permissions};
use std::collections::HashMap;
use crate::config::{ConfigFile, GuestOS, Arch, BootType, Display, DiskImage, Image, Network, PortForward, PreAlloc, Keyboard, SerdeMonitor, SoundCard, Mouse, USBController, Resolution};
use crate::config_parse::{size_unit, parse_optional_bool, BYTES_PER_GB};
use crate::{Args, CliArgs, handle_disk_paths};
use std::os::unix::fs::PermissionsExt;

impl Snapshot {
    pub fn perform_action(&self, conf_data: Vec<String>) -> Result<String> {
        let qemu_img = which("qemu-img").map_err(|_| anyhow!("qemu-img could not be found. Please verify that QEMU is installed on your system."))?;
        let (conf_file, mut conf_data) = crate::parse_conf(conf_data)?;

        let conf_file_path = PathBuf::from(&conf_file);
        let conf_file_path = conf_file_path.parent()
            .ok_or_else(|| anyhow!("The parent directory of the config file cannot be found"))?;

        handle_disk_paths(&mut conf_data.disk_images, conf_file_path)?;
        conf_data.disk_images.into_iter().map(|disk| {
            let disk_img = disk.path;
            match self {
                Self::Apply(name) => match snapshot_command(&qemu_img, "-a", name, &disk_img) {
                    Ok(_) => Ok("Successfully applied snapshot ".to_string() + name + " to " + &disk_img.to_string_lossy()),
                    Err(e) => bail!("Failed to apply snapshot {} to {}: {}", name, &disk_img.to_string_lossy(), e),
                },
                Self::Create(name) => match snapshot_command(&qemu_img, "-c", name, &disk_img) {
                    Ok(_) => Ok("Successfully created snapshot ".to_string() + name + " of " + &disk_img.to_string_lossy()),
                    Err(e) => bail!("Failed to create snapshot {} of {}: {}", name, &disk_img.to_string_lossy(), e),
                },
                Self::Delete(name) => match snapshot_command(&qemu_img, "-d", name, &disk_img) {
                    Ok(_) => Ok("Successfully deleted snapshot ".to_string() + name + " of" + &disk_img.to_string_lossy()),
                    Err(e) => bail!("Failed to delete snapshot {} of {}: {}", name, &disk_img.to_string_lossy(), e),
                },
                Self::Info => {
                    let command = Command::new(&qemu_img).arg("info").arg(&disk_img).output()?;
                    Ok(String::from_utf8_lossy(&command.stdout).to_string() + &String::from_utf8_lossy(&command.stderr))
                }
            }
        }).collect::<Result<Vec<String>>>().map(|v| v.join("\n"))
    }
}

fn snapshot_command(qemu_img: &Path, arg: &str, tag: &str, disk_img: &Path) -> Result<()> {
    let command = Command::new(qemu_img)
        .arg("snapshot")
        .arg(arg)
        .arg(tag)
        .arg(disk_img)
        .output()?;
    if command.status.success() {
        Ok(())
    } else {
        bail!("{}", String::from_utf8_lossy(&command.stderr))
    }
}

pub fn migrate_config(config: Vec<String>) -> Result<String> {
    if config.len() != 2 {
        bail!("Invalid arguments for migrate-config. Usage: `quickemu-rs --migrate-config <config.conf> <config.toml>`")
    }
    let (legacy_conf, toml_conf) = (PathBuf::from(&config[0]), PathBuf::from(&config[1]));
    if legacy_conf.extension().unwrap_or_default() != "conf" || !legacy_conf.exists() {
        bail!("Invalid legacy config file. Please provide a valid .conf file.");
    } else if toml_conf.extension().unwrap_or_default() != "toml" {
        bail!("The configuration file must be migrated to a .toml file.");
    } else if toml_conf.exists() {
        bail!("The target configuration file already exists. Please delete it or provide a new file name.");
    }

    let conf = read_to_string(&legacy_conf).map_err(|e| anyhow!("Could not read legacy configuration file {}: {}", legacy_conf.display(), e))?;
    log::debug!("Legacy configuration: {}", conf);

    let mut conf: HashMap<String, String> = conf.lines().filter_map(|line| {
        log::debug!("Parsing line: {}", line);
        if line.starts_with('#') || !line.contains('=') {
            return None;
        }
        let split = line.split_once('=').unwrap();
        Some((split.0.to_string(), split.1.trim_matches('"').to_string()))
    }).collect::<HashMap<String, String>>();

    let guest_os: GuestOS = (conf.remove("guest_os"), conf.remove("macos_release")).try_into()?;
    let arch: Arch = conf.remove("arch").try_into()?;
    let accelerated = parse_optional_bool(conf.remove("accelerated"), true)?;
    let boot_type: BootType  = (conf.remove("boot"), conf.remove("secureboot")).try_into()?;
    let cpu_cores  = conf.remove("cpu_cores")
        .map(|cores| cores.parse::<std::num::NonZeroUsize>()
        .map_err(|_| anyhow!("Invalid value for cpu_cores: {}", cores))).transpose()?;
    let display: Display = conf.remove("display").try_into()?;
    let network: Network = (conf.remove("network"), conf.remove("macaddr")).try_into()?;
    let ram = conf.get("ram").map(|ram| size_unit(ram)).transpose()?;
    let tpm = parse_optional_bool(conf.remove("tpm"), false)?;
    let keyboard: Keyboard = conf.remove("keyboard").try_into()?;
    let keyboard_layout = conf.remove("keyboard_layout");
    let monitor: SerdeMonitor = (conf.remove("monitor"), conf.remove("monitor_telnet_host"), conf.remove("monitor_telnet_port")).try_into()?;
    let serial: SerdeMonitor = (conf.remove("serial"), conf.remove("serial_telnet_host"), conf.remove("serial_telnet_port")).try_into()?;
    let soundcard: SoundCard = conf.remove("soundcard").try_into()?;
    let resolution: Resolution = conf.remove("resolution").try_into()?;
    let port_forwards = port_forwards(conf.remove("port_forwards"))?;
    let public_dir = conf.remove("public_dir");

    let spice_port = conf.remove("spice_port").map(|port| port.parse::<u16>()
        .map_err(|_| anyhow!("Invalid spice port number: {}", port))).transpose()?.unwrap_or(5930);
    let ssh_port = conf.remove("ssh_port").map(|port| port.parse::<u16>()
        .map_err(|_| anyhow!("Invalid ssh port number: {}", port))).transpose()?.unwrap_or(22220);
    let usb_devices: Option<Vec<String>> = conf.remove("usb_devices")
        .map(|devices| devices.split_whitespace()
            .map(|device| device.trim_matches(['(', ')', ',', ' ', '"']).to_string()
        ).collect());

    let mouse: Option<Mouse> = conf.remove("mouse").map(|mouse| Ok(match mouse.as_str() {
        "usb" => Mouse::Usb,
        "ps2" => Mouse::PS2,
        "virtio" => Mouse::Virtio,
        _ => bail!("Invalid mouse type: {}", mouse),
    })).transpose()?;
    let usb_controller: Option<USBController> = conf.remove("usb_controller").map(|controller| Ok(match controller.as_str() {
        "none" => USBController::None,
        "ehci" => USBController::Ehci,
        "xhci" => USBController::Xhci,
        _ => bail!("Invalid USB controller: {}", controller),
    })).transpose()?;

    let disk_images = {
        let size = conf.get("disk_size").map(|size| size_unit(size)).transpose()?;
        let preallocation: PreAlloc = conf.remove("prealloc").try_into()?;
        let path = PathBuf::from(conf.remove("disk_img").ok_or_else(|| anyhow!("Your legacy configuration file must include a disk_img"))?);
        vec![DiskImage { path, size, preallocation }]
    };
    let image_files: Vec<Image> = [conf.remove("floppy").map(|path| Image::Floppy(PathBuf::from(path))),
        conf.remove("fixed_iso").map(|path| Image::FixedIso(PathBuf::from(path))),
        conf.remove("iso").map(|path| Image::Iso(PathBuf::from(path))),
        conf.remove("img").map(|path| Image::Img(PathBuf::from(path)))]
        .into_iter().flatten().collect();
    let image_files = if image_files.is_empty() { None } else { Some(image_files) };
    if !conf.is_empty() {
        log::warn!("Ignoring values: {:?}", conf);
    }
    
    let config = ConfigFile {
        guest_os, arch, boot_type, cpu_cores, display, disk_images, accelerated, image_files, network, port_forwards, public_dir, ram, tpm, keyboard, keyboard_layout, monitor, serial, soundcard, mouse, resolution, usb_controller, spice_port, ssh_port, usb_devices
    };

    log::debug!("Migrated configuration: {:?}", config);
    let executable = "#!".to_string() + &std::env::current_exe().unwrap_or_default().to_string_lossy() + " --vm\n";
    let toml = executable + &toml::to_string_pretty(&config)
        .map_err(|e| anyhow!("Could not serialize configuration to TOML: {}", e))?;
    log::debug!("TOML: {}", toml);


    match write(&toml_conf, toml.as_bytes()) {
        Ok(_) => {
            set_permissions(&toml_conf, PermissionsExt::from_mode(0o755)).unwrap_or_else(|e| {
                log::warn!("Could not make the TOML configuration file executable: {}", e);
            });
            Ok(format!("Successfully migrated configuration file {} to {}", legacy_conf.display(), toml_conf.display()))
        },
        Err(e) => bail!("Could not write to TOML configuration file {}: {}", toml_conf.display(), e),
    }
}

pub fn port_forwards(bash_array: Option<String>) -> Result<Option<Vec<PortForward>>> {
    match bash_array {
        Some(array) => {
            let ports = array.split_whitespace().filter_map(|pair| pair.trim_matches(['(', ')', ',', ' ', '"']).split_once(':'));
            ports.map(|(host, guest)| {
                Ok(Some(PortForward { host: host.parse::<u16>()?, guest: guest.parse::<u16>()? }))
            }).collect()
        },
        None => Ok(None),
    }
}
impl CliArgs {
    pub fn delete_vm(self) -> Result<()> {
        if get_confirmation("This will delete all files related to the VM {}. Are you sure you want to proceed? (y/N): ")? {
            let (conf_file, _) = crate::parse_conf(self.config_file)
                .map_err(|e| anyhow!("Unable to delete VM due to error in configuration file: {}", e))?;
            let vm_dir = conf_file[..conf_file.len()-5].parse::<PathBuf>()?;

            std::fs::remove_file(&conf_file)
                .map_err(|e| anyhow!("Unable to remove config file {}: {}", conf_file, e))?;
            std::fs::remove_dir_all(&vm_dir)
                .inspect(|_| println!("Deleted {} and {}", conf_file, vm_dir.display()))
                .map_err(|e| anyhow!("Failed to delete VM dir {}: {}", vm_dir.display(), e))
        } else {
            println!("Cancelled deletion.");
            Ok(())
        }
    }
}

impl Args {
    pub fn delete_disk(&self) -> Result<()> {
        let disk_list = self.disk_images.iter().map(|disk| format!("Path: {}, Configured size: {} GiB, Current size: {} GiB, Preallocation: {}", 
            disk.path.display(), disk.size.unwrap_or(self.guest_os.disk_size()) as f64 / BYTES_PER_GB as f64, disk.path.metadata().map(|data| data.len()).unwrap_or_default() as f64 / BYTES_PER_GB as f64, disk.preallocation))
            .collect::<Vec<String>>().join("\n");

        if get_confirmation(&format!("This will delete the VM's OVMF VARS along with the following disks\n{disk_list}\nAre you sure you want to proceed? (y/N): "))? {
            let vars = self.vm_dir.join("OVMF_VARS.fd");
            if vars.exists() {
                std::fs::remove_file(&vars)
                    .map_err(|e| anyhow!("Unable to delete OVMF VARS file {}: {}", vars.display(), e))?;
                println!("Deleted OVMF VARS: {}", vars.display());
            }
            for disk in &self.disk_images {
                let path = &disk.path;
                std::fs::remove_file(path)
                    .map_err(|e| anyhow!("Unable to delete disk {}: {}", path.display(), e))?;
                println!("Deleted disk: {}", path.display());
            };
        } else {
            println!("Cancelled deletion.");
        }
        Ok(())
    }
}

fn get_confirmation(prompt: &str) -> Result<bool> {
    print!("{prompt}");
    std::io::stdout().flush()?;
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    input = input.trim().to_ascii_lowercase();

    match input.as_str() {
        "yes" | "y" => Ok(true),
        "no" | "n" | "" => Ok(false),
        invalid => bail!("Invalid input: {}", invalid),
    }
}
