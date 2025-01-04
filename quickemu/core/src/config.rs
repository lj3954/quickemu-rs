use crate::{
    data::*,
    error::{ConfigError, MonitorError},
    utils::EmulatorArgs,
};
use itertools::chain;
use serde::{Deserialize, Serialize};
use std::{
    iter,
    path::{Path, PathBuf},
};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub vm_dir: Option<PathBuf>,
    pub guest: GuestOS,
    #[serde(default, skip_serializing_if = "is_default")]
    pub machine: MachineInfo,
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

    fn to_arg_iter(self) -> impl Iterator<Item = impl EmulatorArgs> {
        todo!()
    }
}
