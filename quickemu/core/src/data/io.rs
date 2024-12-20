use super::is_default;
use clap::ValueEnum;
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Io {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usb_controller: Option<USBController>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub usb_devices: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keyboard: Option<Keyboard>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub keyboard_layout: KeyboardLayout,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mouse: Option<Mouse>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub soundcard: Option<SoundCard>,
}

#[derive(PartialEq, ValueEnum, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum USBController {
    None,
    Ehci,
    Xhci,
}

#[derive(PartialEq, ValueEnum, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Mouse {
    Usb,
    Tablet,
    Virtio,
    PS2,
}

#[derive(Default, Copy, ValueEnum, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum SoundCard {
    None,
    #[default]
    IntelHDA,
    AC97,
    ES1370,
    SB16,
}
#[derive(Default, ValueEnum, PartialEq, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Keyboard {
    #[default]
    Usb,
    Virtio,
    PS2,
}

#[derive(Display, PartialEq, Default, ValueEnum, Clone, Debug, Serialize, Deserialize)]
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
