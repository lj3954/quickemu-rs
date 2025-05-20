use std::str::FromStr;

use clap::{Parser, ValueEnum};
use quickemu_core::data::{self, Keyboard, KeyboardLayout, Mouse, PublicDir, SoundCard, USBController};

#[derive(Debug, Parser)]
pub(crate) struct IoArgs {
    #[clap(flatten)]
    display: super::display::DisplayArgs,
    #[clap(long, display_order = 1)]
    keyboard: Option<CliKeyboard>,
    #[clap(long, display_order = 1)]
    keyboard_layout: Option<CliKeyboardLayout>,
    #[clap(long, display_order = 1)]
    mouse: Option<CliMouse>,
    #[clap(long, display_order = 1)]
    soundcard: Option<CliSoundCard>,
    #[clap(long, display_order = 1, value_parser = PublicDir::from_str, help = "A directory, 'default', or 'none'")]
    public_dir: Option<PublicDir>,
    #[clap(long, display_order = 1)]
    usb_controller: Option<CliUsbController>,
}

impl IoArgs {
    pub(crate) fn edit_config(self, config: &mut data::Io) {
        if let Some(keyboard) = self.keyboard {
            config.keyboard = Some(keyboard.into());
        }
        if let Some(keyboard_layout) = self.keyboard_layout {
            config.keyboard_layout = keyboard_layout.into();
        }
        if let Some(mouse) = self.mouse {
            config.mouse = Some(mouse.into());
        }
        if let Some(soundcard) = self.soundcard {
            config.soundcard = Some(soundcard.into());
        }
        if let Some(usb_controller) = self.usb_controller {
            config.usb_controller = Some(usb_controller.into());
        }
        if let Some(public_dir) = self.public_dir {
            config.public_dir = public_dir;
        }
        self.display.edit_config(&mut config.display);
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum CliKeyboard {
    Usb,
    Virtio,
    PS2,
}

impl From<CliKeyboard> for Keyboard {
    fn from(value: CliKeyboard) -> Self {
        match value {
            CliKeyboard::Usb => Keyboard::Usb,
            CliKeyboard::Virtio => Keyboard::Virtio,
            CliKeyboard::PS2 => Keyboard::PS2,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
pub enum CliKeyboardLayout {
    Arabic,
    SwissGerman,
    Spanish,
    Faroese,
    FrenchCanadian,
    Hungarian,
    Japanese,
    Macedonian,
    Norwegian,
    BrazilianPortuguese,
    Swedish,
    Danish,
    BritishEnglish,
    Estonian,
    French,
    SwissFrench,
    Icelandic,
    Lithuanian,
    Dutch,
    Polish,
    Russian,
    Thai,
    German,
    AmericanEnglish,
    Finnish,
    BelgianFrench,
    Croatian,
    Italian,
    Latvian,
    NorwegianBokmal,
    Portuguese,
    Slovenian,
    Turkish,
}

impl From<CliKeyboardLayout> for KeyboardLayout {
    fn from(value: CliKeyboardLayout) -> Self {
        match value {
            CliKeyboardLayout::Arabic => KeyboardLayout::Arabic,
            CliKeyboardLayout::SwissGerman => KeyboardLayout::SwissGerman,
            CliKeyboardLayout::Spanish => KeyboardLayout::Spanish,
            CliKeyboardLayout::Faroese => KeyboardLayout::Faroese,
            CliKeyboardLayout::FrenchCanadian => KeyboardLayout::FrenchCanadian,
            CliKeyboardLayout::Hungarian => KeyboardLayout::Hungarian,
            CliKeyboardLayout::Japanese => KeyboardLayout::Japanese,
            CliKeyboardLayout::Macedonian => KeyboardLayout::Macedonian,
            CliKeyboardLayout::Norwegian => KeyboardLayout::Norwegian,
            CliKeyboardLayout::BrazilianPortuguese => KeyboardLayout::BrazilianPortuguese,
            CliKeyboardLayout::Swedish => KeyboardLayout::Swedish,
            CliKeyboardLayout::Danish => KeyboardLayout::Danish,
            CliKeyboardLayout::BritishEnglish => KeyboardLayout::BritishEnglish,
            CliKeyboardLayout::Estonian => KeyboardLayout::Estonian,
            CliKeyboardLayout::French => KeyboardLayout::French,
            CliKeyboardLayout::SwissFrench => KeyboardLayout::SwissFrench,
            CliKeyboardLayout::Icelandic => KeyboardLayout::Icelandic,
            CliKeyboardLayout::Lithuanian => KeyboardLayout::Lithuanian,
            CliKeyboardLayout::Dutch => KeyboardLayout::Dutch,
            CliKeyboardLayout::Polish => KeyboardLayout::Polish,
            CliKeyboardLayout::Russian => KeyboardLayout::Russian,
            CliKeyboardLayout::Thai => KeyboardLayout::Thai,
            CliKeyboardLayout::German => KeyboardLayout::German,
            CliKeyboardLayout::AmericanEnglish => KeyboardLayout::AmericanEnglish,
            CliKeyboardLayout::Finnish => KeyboardLayout::Finnish,
            CliKeyboardLayout::BelgianFrench => KeyboardLayout::BelgianFrench,
            CliKeyboardLayout::Croatian => KeyboardLayout::Croatian,
            CliKeyboardLayout::Italian => KeyboardLayout::Italian,
            CliKeyboardLayout::Latvian => KeyboardLayout::Latvian,
            CliKeyboardLayout::NorwegianBokmal => KeyboardLayout::NorwegianBokmal,
            CliKeyboardLayout::Portuguese => KeyboardLayout::Portuguese,
            CliKeyboardLayout::Slovenian => KeyboardLayout::Slovenian,
            CliKeyboardLayout::Turkish => KeyboardLayout::Turkish,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum CliMouse {
    Usb,
    Tablet,
    Virtio,
    PS2,
}

impl From<CliMouse> for Mouse {
    fn from(value: CliMouse) -> Self {
        match value {
            CliMouse::Usb => Mouse::Usb,
            CliMouse::Tablet => Mouse::Tablet,
            CliMouse::Virtio => Mouse::Virtio,
            CliMouse::PS2 => Mouse::PS2,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum CliSoundCard {
    None,
    IntelHDA,
    AC97,
    ES1370,
    SB16,
    USBAudio,
}

impl From<CliSoundCard> for SoundCard {
    fn from(value: CliSoundCard) -> Self {
        match value {
            CliSoundCard::None => SoundCard::None,
            CliSoundCard::IntelHDA => SoundCard::IntelHDA,
            CliSoundCard::AC97 => SoundCard::AC97,
            CliSoundCard::ES1370 => SoundCard::ES1370,
            CliSoundCard::SB16 => SoundCard::SB16,
            CliSoundCard::USBAudio => SoundCard::USBAudio,
        }
    }
}

#[derive(Debug, Clone, ValueEnum)]
enum CliUsbController {
    None,
    Ehci,
    Xhci,
}

impl From<CliUsbController> for USBController {
    fn from(value: CliUsbController) -> Self {
        match value {
            CliUsbController::None => USBController::None,
            CliUsbController::Ehci => USBController::Ehci,
            CliUsbController::Xhci => USBController::Xhci,
        }
    }
}
