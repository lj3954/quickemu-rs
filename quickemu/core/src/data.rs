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

use serde::de;
use std::fmt;

pub fn is_default<T: Default + PartialEq>(input: &T) -> bool {
    input == &T::default()
}
pub fn is_true(input: &bool) -> bool {
    *input
}

fn parse_size<E: de::Error>(value: &str) -> Result<u64, E> {
    let mut chars = value.chars().rev();
    let mut unit_char = chars.next();
    if unit_char.map_or(false, |c| c == 'B') {
        unit_char = chars.next();
    }
    let unit_char = unit_char.ok_or_else(|| de::Error::custom("No unit type was specified"))?;
    let size = match unit_char {
        'K' => 1u64 << 10,
        'M' => 1 << 20,
        'G' => 1 << 30,
        'T' => 1 << 40,
        _ => return Err(de::Error::custom("Unexpected unit type")),
    } as f64;

    let rem: String = chars.rev().collect();
    let size_f: f64 = rem.parse().map_err(de::Error::custom)?;
    Ok((size_f * size) as u64)
}

struct SizeUnit;
impl de::Visitor<'_> for SizeUnit {
    type Value = Option<u64>;
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string (ending in a size unit, e.g. M, G, T) or a number (in bytes)")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        parse_size(value).map(Some)
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
