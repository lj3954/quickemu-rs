use anyhow::{anyhow, bail, Result};
use crate::config::Snapshot;
use which::which;
use std::process::Command;
use std::path::{Path, PathBuf};

impl Snapshot {
    pub fn perform_action(&self, conf_data: Vec<String>) -> Result<String> {
        let qemu_img = which("qemu-img").map_err(|_| anyhow!("qemu-img could not be found. Please verify that QEMU is installed on your system."))?;
        let (conf_file, mut conf_data) = crate::parse_conf(conf_data)?;
        let conf_file_path = PathBuf::from(&conf_file)
            .canonicalize()?
            .parent()
            .ok_or_else(|| anyhow!("The parent directory of the config file cannot be found"))?
            .to_path_buf();
        crate::handle_disk_paths(&mut conf_data.disk_images, &conf_file_path)?;
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
