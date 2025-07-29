#[cfg(feature = "quickemu")]
use std::path::Path;
use std::path::PathBuf;

use super::{is_default, Display};
use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Default, Clone, Debug, PartialEq, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "is_default")]
    pub public_dir: PublicDir,
}

impl Io {
    #[cfg(feature = "quickemu")]
    pub(crate) fn public_dir(&self) -> Option<&Path> {
        self.public_dir.as_ref().as_deref()
    }
}

#[derive(PartialEq, Default, Debug, Deserialize, Serialize, derive_more::AsRef, Clone)]
pub struct USBDevices(Option<Vec<String>>);

#[cfg_attr(not(feature = "quickemu"), derive(Default))]
#[derive(PartialEq, Clone, Debug, Deserialize, Serialize, derive_more::AsRef)]
pub struct PublicDir(Option<PathBuf>);

#[cfg(feature = "quickemu")]
impl Default for PublicDir {
    fn default() -> Self {
        let public_dir = dirs::public_dir();
        let home_dir = dirs::home_dir();

        // If the default public dir is the user's home directory, we won't share it with the guest
        // for security reasons
        if home_dir != public_dir {
            Self(public_dir)
        } else {
            Self(None)
        }
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
                    return Err(serde::de::Error::custom(format!("Path '{value}' is not a directory")));
                }
                Ok(Self(Some(path)))
            }
        }
    }
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum USBController {
    None,
    #[serde(alias = "EHCI")]
    Ehci,
    #[serde(alias = "XHCI")]
    Xhci,
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mouse {
    #[serde(alias = "USB")]
    Usb,
    Tablet,
    Virtio,
    #[serde(alias = "PS2")]
    PS2,
}

#[derive(Copy, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SoundCard {
    None,
    #[serde(alias = "Intel HDA")]
    IntelHDA,
    #[serde(alias = "AC97")]
    AC97,
    #[serde(alias = "ES1370")]
    ES1370,
    #[serde(alias = "SB16")]
    SB16,
    #[serde(alias = "USB Audio")]
    USBAudio,
}

#[derive(Copy, Default, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Keyboard {
    #[default]
    #[serde(alias = "USB")]
    Usb,
    Virtio,
    #[serde(alias = "PS2")]
    PS2,
}

#[derive(derive_more::Display, Copy, PartialEq, Default, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
