use crate::data::*;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct ConfigFile {
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
