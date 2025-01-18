use super::{default_if_empty, is_default};
use serde::{de::Visitor, Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr};

#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Display {
    #[serde(default, flatten, rename = "type")]
    #[serde(deserialize_with = "default_if_empty")]
    pub display_type: DisplayType,
    #[serde(default, skip_serializing_if = "is_default")]
    pub resolution: Resolution,
    pub accelerated: Accelerated,
    #[serde(default, skip_serializing_if = "is_default")]
    pub braille: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Accelerated(bool);

impl From<Accelerated> for bool {
    fn from(value: Accelerated) -> Self {
        value.0
    }
}

impl AsRef<str> for Accelerated {
    fn as_ref(&self) -> &str {
        if self.0 {
            "on"
        } else {
            "off"
        }
    }
}

impl std::fmt::Display for Accelerated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = if self.0 { "Enabled" } else { "Disabled" };
        write!(f, "{}", text)
    }
}

impl Default for Accelerated {
    fn default() -> Self {
        Self(default_accel())
    }
}
fn default_accel() -> bool {
    cfg!(not(target_os = "macos"))
}

impl Visitor<'_> for Accelerated {
    type Value = Accelerated;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a boolean")
    }
    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self(value))
    }
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Resolution {
    #[default]
    Default,
    #[cfg(feature = "display_resolution")]
    Display {
        display_name: Option<String>,
        percentage: Option<f64>,
    },
    Custom {
        width: u32,
        height: u32,
    },
    FullScreen,
}

#[derive(Copy, Default, derive_more::Display, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DisplayType {
    #[serde(alias = "none")]
    None,
    #[serde(alias = "sdl", alias = "SDL")]
    #[display("SDL")]
    #[cfg_attr(not(target_os = "macos"), default)]
    Sdl,
    #[serde(alias = "gtk", alias = "GTK")]
    #[display("GTK")]
    Gtk,
    #[cfg(not(target_os = "macos"))]
    #[display("Spice")]
    #[serde(alias = "spice")]
    Spice {
        #[serde(default, skip_serializing_if = "is_default")]
        access: Access,
        #[serde(default, skip_serializing_if = "is_default")]
        viewer: Viewer,
        #[serde(default = "default_spice_port", skip_serializing_if = "is_default_spice")]
        spice_port: u16,
    },
    #[cfg(not(target_os = "macos"))]
    #[serde(alias = "spice_app", alias = "spice-app")]
    #[display("Spice App")]
    SpiceApp,
    #[cfg(target_os = "macos")]
    #[cfg_attr(target_os = "macos", default)]
    #[serde(alias = "cocoa")]
    Cocoa,
}
const fn default_spice_port() -> u16 {
    5930
}
fn is_default_spice(input: &u16) -> bool {
    *input == default_spice_port()
}

#[cfg(not(target_os = "macos"))]
#[derive(Copy, derive_more::Display, PartialEq, Default, Deserialize, Serialize, Clone, Debug)]
pub enum Viewer {
    None,
    #[default]
    Spicy,
    Remote,
}

#[cfg(not(target_os = "macos"))]
#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize, derive_more::AsRef)]
pub struct Access(Option<IpAddr>);

#[cfg(not(target_os = "macos"))]
impl Default for Access {
    fn default() -> Self {
        local_access()
    }
}

#[cfg(not(target_os = "macos"))]
fn local_access() -> Access {
    Access(Some(IpAddr::V4(Ipv4Addr::LOCALHOST)))
}

#[cfg(not(target_os = "macos"))]
impl Visitor<'_> for Access {
    type Value = Access;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an IP address, 'remote', or 'local'")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(match value {
            "remote" => Self(None),
            "local" => local_access(),
            _ => {
                let address = value.parse().map_err(serde::de::Error::custom)?;
                Self(Some(address))
            }
        })
    }
}
