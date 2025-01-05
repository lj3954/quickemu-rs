use crate::{
    data::*,
    error::{ConfigError, Error, MonitorError, Warning},
    qemu_args,
    utils::{EmulatorArgs, QemuArg},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub vm_dir: Option<PathBuf>,
    pub guest: GuestOS,
    #[serde(default, skip_serializing_if = "is_default")]
    pub machine: Machine,
    #[serde(default, skip_serializing_if = "is_default")]
    pub display: Display,
    pub disk_images: Vec<DiskImage>,
    pub image_files: Option<Vec<Image>>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub network: Network,
    #[serde(default, skip_serializing_if = "is_default")]
    pub io: Io,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
}

impl Config {
    pub fn parse(file: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(file)?;
        toml::from_str(&contents).map_err(ConfigError::Parse)
    }

    pub fn send_monitor_command(&self, command: &str) -> Result<String, MonitorError> {
        self.network.send_monitor_cmd(command)
    }

    pub fn to_qemu_args(&self) -> Result<(Vec<QemuArg>, Vec<Warning>), Error> {
        qemu_args!(self.machine.cpu_args(self.guest), self.guest.tweaks(self.machine.arch))
    }
}
