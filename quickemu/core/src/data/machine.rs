use super::{deserialize_size, is_default};
use derive_more::derive::Display;
use serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize, Debug, PartialEq)]
pub struct Machine {
    pub cpu_threads: Option<std::num::NonZeroUsize>,
    #[serde(default, flatten)]
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

#[derive(Display, Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "arch")]
pub enum Arch {
    #[serde(rename = "x86_64")]
    X86_64 {
        #[serde(default, skip_serializing_if = "is_default")]
        machine: X86_64Machine,
    },
    #[serde(alias = "aarch64")]
    AArch64 {
        #[serde(default, skip_serializing_if = "is_default")]
        machine: AArch64Machine,
    },
    #[serde(rename = "riscv64")]
    Riscv64 {
        #[serde(default, skip_serializing_if = "is_default")]
        machine: Riscv64Machine,
    },
}

impl Default for Arch {
    fn default() -> Self {
        Self::X86_64 { machine: X86_64Machine::Standard }
    }
}

// Below enums will be used for future SBC / other specialized machine emulation
#[derive(Display, Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum X86_64Machine {
    #[default]
    Standard,
}

#[derive(Display, Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AArch64Machine {
    #[default]
    Standard,
}

#[derive(Display, Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Riscv64Machine {
    #[default]
    Standard,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum BootType {
    #[serde(alias = "EFI", alias = "Efi")]
    Efi {
        #[serde(default)]
        secure_boot: bool,
    },
    #[serde(alias = "Legacy", alias = "bios", alias = "BIOS")]
    Legacy,
}
impl Default for BootType {
    fn default() -> Self {
        Self::Efi { secure_boot: false }
    }
}
