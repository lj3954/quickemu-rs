pub mod display;
pub mod guest;
pub mod image;
pub mod io;
pub mod machine;
pub mod network;

pub use display::*;
pub use guest::*;
pub use image::*;
pub use io::*;
pub use machine::*;
pub use network::*;

use serde::{de, Deserialize, Serialize};
use std::{fmt, path::PathBuf};

pub fn is_default<T: Default + PartialEq>(input: &T) -> bool {
    input == &T::default()
}
pub fn is_true(input: &bool) -> bool {
    *input
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct DiskImage {
    pub path: PathBuf,
    #[serde(deserialize_with = "deserialize_size", default)]
    pub size: Option<u64>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub preallocation: PreAlloc,
    pub format: Option<DiskFormat>,
}

struct SizeUnit;
impl<'de> de::Visitor<'de> for SizeUnit {
    type Value = Option<u64>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string (ending in a size unit, e.g. M, G, T) or a number (in bytes)")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let mut chars = value.chars().rev();
        let mut unit_char = chars.next();
        if unit_char.map_or(false, |c| c == 'B') {
            unit_char = chars.next();
        }
        let unit_char = unit_char.ok_or_else(|| de::Error::custom("No unit type was specified"))?;
        let size = match unit_char {
            'K' => 2u64 << 9,
            'M' => 2 << 19,
            'G' => 2 << 29,
            'T' => 2 << 39,
            _ => return Err(de::Error::custom("Unexpected unit type")),
        } as f64;

        let rem: String = chars.rev().collect();
        let size_f: f64 = rem.parse().map_err(de::Error::custom)?;
        Ok(Some((size_f * size) as u64))
    }
    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Some(value.try_into().map_err(serde::de::Error::custom)?))
    }
}
pub fn deserialize_size<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserializer.deserialize_any(SizeUnit)
}
