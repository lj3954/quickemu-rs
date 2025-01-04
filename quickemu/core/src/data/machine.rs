use super::{deserialize_size, is_default};
use clap::ValueEnum;
use derive_more::derive::Display;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct MachineInfo {
    pub cpu_threads: Option<std::num::NonZeroUsize>,
    #[serde(default)]
    pub arch: Arch,
    #[serde(default, skip_serializing_if = "is_default")]
    pub boot: BootType,
    #[serde(default, skip_serializing_if = "is_default")]
    pub tpm: bool,
    #[serde(deserialize_with = "deserialize_size", default)]
    pub ram: Option<u64>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub status_quo: bool,
}

#[allow(non_camel_case_types)]
#[derive(Display, ValueEnum, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[clap(rename_all = "verbatim")]
pub enum Arch {
    #[default]
    x86_64,
    aarch64,
    riscv64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BootType {
    #[serde(rename = "efi", alias = "EFI", alias = "Efi")]
    Efi {
        #[serde(default)]
        secure_boot: bool,
    },
    #[serde(rename = "legacy", alias = "Legacy", alias = "bios")]
    Legacy,
}
impl Default for BootType {
    fn default() -> Self {
        Self::Efi { secure_boot: false }
    }
}
