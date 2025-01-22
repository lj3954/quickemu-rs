use std::path::{Path, PathBuf};

use super::{is_default, Display};
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Io {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usb_controller: Option<USBController>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<Keyboard>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub keyboard_layout: KeyboardLayout,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mouse: Option<Mouse>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub soundcard: Option<SoundCard>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub display: Display,
    pub public_dir: PublicDir,
}

impl Io {
    pub(crate) fn public_dir(&self) -> Option<&Path> {
        self.public_dir.as_ref().as_deref()
    }
}

#[derive(PartialEq, Default, Debug, Deserialize, Serialize, derive_more::AsRef, Clone)]
pub struct USBDevices(Option<Vec<String>>);

#[derive(PartialEq, Debug, Deserialize, Serialize, derive_more::AsRef)]
pub struct PublicDir(Option<PathBuf>);

impl Default for PublicDir {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_default();
        let public = dirs::public_dir().unwrap_or_default();
        Self((home != public).then_some(public))
    }
}

impl Visitor<'_> for PublicDir {
    type Value = PublicDir;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a valid path, 'default', or 'none'")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match value {
            "default" => Ok(Self::default()),
            "none" => Ok(Self(None)),
            _ => {
                let path = PathBuf::from(value);
                if !path.is_dir() {
                    return Err(serde::de::Error::custom(format!("Path '{}' is not a directory", value)));
                }
                Ok(Self(Some(path)))
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum USBController {
    None,
    Ehci,
    Xhci,
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Mouse {
    Usb,
    Tablet,
    Virtio,
    PS2,
}

#[derive(Copy, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum SoundCard {
    None,
    IntelHDA,
    AC97,
    ES1370,
    SB16,
    USBAudio,
}

#[derive(Copy, Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Keyboard {
    #[default]
    Usb,
    Virtio,
    PS2,
}

#[derive(derive_more::Display, Copy, PartialEq, Default, Clone, Debug, Serialize, Deserialize)]
pub enum KeyboardLayout {
    #[serde(alias = "ar")]
    Arabic,
    #[serde(alias = "de-ch")]
    SwissGerman,
    #[serde(alias = "es")]
    Spanish,
    #[serde(alias = "fo")]
    Faroese,
    #[serde(alias = "fr-ca")]
    FrenchCanadian,
    #[serde(alias = "hu")]
    Hungarian,
    #[serde(alias = "ja")]
    Japanese,
    #[serde(alias = "mk")]
    Macedonian,
    #[serde(alias = "no")]
    Norwegian,
    #[serde(alias = "pt-br")]
    BrazilianPortuguese,
    #[serde(alias = "sv")]
    Swedish,
    #[serde(alias = "da")]
    Danish,
    #[serde(alias = "en-gb")]
    BritishEnglish,
    #[serde(alias = "et")]
    Estonian,
    #[serde(alias = "fr")]
    French,
    #[serde(alias = "fr-ch")]
    SwissFrench,
    #[serde(alias = "is")]
    Icelandic,
    #[serde(alias = "lt")]
    Lithuanian,
    #[serde(alias = "nl")]
    Dutch,
    #[serde(alias = "pl")]
    Polish,
    #[serde(alias = "ru")]
    Russian,
    #[serde(alias = "th")]
    Thai,
    #[serde(alias = "de")]
    German,
    #[serde(alias = "en-us")]
    #[default]
    AmericanEnglish,
    #[serde(alias = "fi")]
    Finnish,
    #[serde(alias = "fr-be")]
    BelgianFrench,
    #[serde(alias = "hr")]
    Croatian,
    #[serde(alias = "it")]
    Italian,
    #[serde(alias = "lv")]
    Latvian,
    #[serde(alias = "nb")]
    NorwegianBokmal,
    #[serde(alias = "pt")]
    Portuguese,
    #[serde(alias = "sl")]
    Slovenian,
    #[serde(alias = "tr")]
    Turkish,
}

impl KeyboardLayout {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Arabic => "ar",
            Self::SwissGerman => "de-ch",
            Self::Spanish => "es",
            Self::Faroese => "fo",
            Self::FrenchCanadian => "fr-ca",
            Self::Hungarian => "hu",
            Self::Japanese => "ja",
            Self::Macedonian => "mk",
            Self::Norwegian => "no",
            Self::BrazilianPortuguese => "pt-br",
            Self::Swedish => "sv",
            Self::Danish => "da",
            Self::BritishEnglish => "en-gb",
            Self::Estonian => "et",
            Self::French => "fr",
            Self::SwissFrench => "fr-ch",
            Self::Icelandic => "is",
            Self::Lithuanian => "lt",
            Self::Dutch => "nl",
            Self::Polish => "pl",
            Self::Russian => "ru",
            Self::Thai => "th",
            Self::German => "de",
            Self::AmericanEnglish => "en-us",
            Self::Finnish => "fi",
            Self::BelgianFrench => "fr-be",
            Self::Croatian => "hr",
            Self::Italian => "it",
            Self::Latvian => "lv",
            Self::NorwegianBokmal => "nb",
            Self::Portuguese => "pt",
            Self::Slovenian => "sl",
            Self::Turkish => "tr",
        }
    }
}
