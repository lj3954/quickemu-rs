use super::{default_if_empty, is_default};
use clap::{builder::PossibleValue, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

#[derive(Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Display {
    #[serde(default, flatten, rename = "type")]
    #[serde(deserialize_with = "default_if_empty")]
    pub display_type: DisplayType,
    #[serde(default, skip_serializing_if = "is_default")]
    pub resolution: Resolution,
    // Serde appears to have a bug where it uses T::Default() rather than the specified default
    // deserializer when a field isn't present. Instead, we'll use an optional value for this
    pub accelerated: Option<bool>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub braille: bool,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Resolution {
    #[default]
    Default,
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
    SpiceApp {
        #[serde(default, skip_serializing_if = "is_default")]
        access: Access,
        #[serde(default, skip_serializing_if = "is_default")]
        viewer: Viewer,
        #[serde(default = "default_spice_port", skip_serializing_if = "is_default_spice")]
        spice_port: u16,
    },
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

const DISPLAY_VARIANTS: &[DisplayType] = &[
    DisplayType::None,
    DisplayType::Sdl,
    DisplayType::Gtk,
    #[cfg(not(target_os = "macos"))]
    DisplayType::Spice {
        access: Access::Local,
        viewer: Viewer::Spicy,
        spice_port: default_spice_port(),
    },
    #[cfg(not(target_os = "macos"))]
    DisplayType::SpiceApp {
        access: Access::Local,
        viewer: Viewer::Spicy,
        spice_port: default_spice_port(),
    },
    #[cfg(target_os = "macos")]
    DisplayType::Cocoa,
];

impl ValueEnum for DisplayType {
    fn value_variants<'a>() -> &'a [Self] {
        &DISPLAY_VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::None => PossibleValue::new("none"),
            Self::Sdl => PossibleValue::new("sdl"),
            Self::Gtk => PossibleValue::new("gtk"),
            #[cfg(not(target_os = "macos"))]
            Self::Spice { .. } => PossibleValue::new("spice"),
            #[cfg(not(target_os = "macos"))]
            Self::SpiceApp { .. } => PossibleValue::new("spice-app"),
            #[cfg(target_os = "macos")]
            Self::Cocoa => PossibleValue::new("cocoa"),
        })
    }
}

impl FromStr for DisplayType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "sdl" => Ok(Self::Sdl),
            "gtk" => Ok(Self::Gtk),
            #[cfg(not(target_os = "macos"))]
            "spice" => Ok(Self::Spice {
                access: Access::Local,
                viewer: Viewer::Spicy,
                spice_port: default_spice_port(),
            }),
            #[cfg(not(target_os = "macos"))]
            "spice-app" => Ok(Self::SpiceApp {
                access: Access::Local,
                viewer: Viewer::Spicy,
                spice_port: default_spice_port(),
            }),
            #[cfg(target_os = "macos")]
            "cocoa" => Ok(Self::Cocoa),
            _ => Err(format!("Invalid variant: {s}")),
        }
    }
}

#[cfg(not(target_os = "macos"))]
#[derive(Copy, derive_more::Display, PartialEq, Default, Deserialize, Serialize, ValueEnum, Clone, Debug)]
pub enum Viewer {
    None,
    #[default]
    Spicy,
    Remote,
}

#[cfg(not(target_os = "macos"))]
#[derive(Copy, Clone, Default, Serialize, Deserialize, Debug, PartialEq)]
pub enum Access {
    Remote,
    #[default]
    Local,
    Address(IpAddr),
}

#[cfg(not(target_os = "macos"))]
const ACCESS_VARIANTS: [Access; 3] = [Access::Remote, Access::Local, Access::Address(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)))];

#[cfg(not(target_os = "macos"))]
impl ValueEnum for Access {
    fn value_variants<'a>() -> &'a [Self] {
        &ACCESS_VARIANTS
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        Some(match self {
            Self::Remote => PossibleValue::new("remote"),
            Self::Local => PossibleValue::new("local"),
            Self::Address(_) => PossibleValue::new("other"),
        })
    }
}

#[cfg(not(target_os = "macos"))]
impl FromStr for Access {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "remote" => Ok(Self::Remote),
            "local" => Ok(Self::Local),
            _ => IpAddr::from_str(s).map(Self::Address).map_err(|e| e.to_string()),
        }
    }
}
