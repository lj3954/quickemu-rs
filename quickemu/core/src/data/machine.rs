use super::{default_if_empty, deserialize_size, is_default};
use derive_more::derive::Display;
use itertools::chain;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Machine {
    pub cpu_threads: Option<std::num::NonZeroUsize>,
    #[serde(default, flatten, deserialize_with = "default_if_empty")]
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
    #[display("x86_64 ({})", machine)]
    #[serde(rename = "x86_64")]
    X86_64 {
        #[serde(default, skip_serializing_if = "is_default")]
        machine: X86_64Machine,
    },
    #[display("AArch64 ({})", machine)]
    #[serde(alias = "aarch64")]
    AArch64 {
        #[serde(default, skip_serializing_if = "is_default")]
        machine: AArch64Machine,
    },
    #[display("Riscv64 ({})", machine)]
    #[serde(rename = "riscv64")]
    Riscv64 {
        #[serde(default, skip_serializing_if = "is_default")]
        machine: Riscv64Machine,
    },
}

impl Arch {
    pub fn iter() -> impl Iterator<Item = Self> {
        chain!(
            X86_64Machine::iter().map(|machine| Self::X86_64 { machine }),
            AArch64Machine::iter().map(|machine| Self::AArch64 { machine }),
            Riscv64Machine::iter().map(|machine| Self::Riscv64 { machine })
        )
    }
}

impl Default for Arch {
    fn default() -> Self {
        Self::X86_64 { machine: X86_64Machine::Standard }
    }
}

// Below enums will be used for future SBC / other specialized machine emulation
#[derive(Display, Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize, EnumIter)]
#[serde(rename_all = "snake_case")]
pub enum X86_64Machine {
    #[default]
    Standard,
}

#[derive(Display, Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize, EnumIter)]
#[serde(rename_all = "snake_case")]
pub enum AArch64Machine {
    #[default]
    Standard,
}

#[derive(Display, Copy, Clone, Default, Debug, PartialEq, Serialize, Deserialize, EnumIter)]
#[serde(rename_all = "snake_case")]
pub enum Riscv64Machine {
    #[default]
    Standard,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
