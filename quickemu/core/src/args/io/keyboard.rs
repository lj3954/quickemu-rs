use std::{borrow::Cow, ffi::OsStr};

use crate::{
    data::{GuestOS, Keyboard, KeyboardLayout},
    utils::{EmulatorArgs, QemuArg},
};

impl GuestOS {
    pub(crate) fn default_keyboard(&self) -> Keyboard {
        match self {
            GuestOS::ReactOS => Keyboard::PS2,
            _ => Keyboard::Usb,
        }
    }
}

impl EmulatorArgs for Keyboard {
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let device = match self {
            Self::PS2 => return vec![],
            Self::Usb => "usb-kbd,bus=input.0",
            Self::Virtio => "virtio-keyboard",
        };
        vec![Cow::Borrowed(OsStr::new("-device")), Cow::Borrowed(OsStr::new(device))]
    }
}

impl EmulatorArgs for KeyboardLayout {
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let layout = match self {
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
        };
        vec![Cow::Borrowed(OsStr::new("-k")), Cow::Borrowed(OsStr::new(layout))]
    }
}
