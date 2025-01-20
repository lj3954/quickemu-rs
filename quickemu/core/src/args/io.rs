use audio::Audio;
use display::DisplayArgs;
use itertools::chain;
use usb::USBArgs;

use crate::{
    data::{Arch, GuestOS, Io, Keyboard, KeyboardLayout, Mouse},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

mod audio;
mod display;
mod keyboard;
mod mouse;
mod usb;

#[cfg(not(target_os = "macos"))]
mod spice;
#[cfg(not(target_os = "macos"))]
use crate::data::DisplayType;

impl<'a> Io {
    pub fn args(&'a self, arch: Arch, guest: GuestOS, vm_name: &'a str) -> Result<(IoArgs<'a>, Vec<Warning>), Error> {
        let mut warnings = Vec::new();

        let keyboard = self.keyboard.unwrap_or(guest.default_keyboard());
        let usb_controller = self.usb_controller.unwrap_or(guest.default_usb_controller());

        let soundcard = self.soundcard.unwrap_or(guest.default_soundcard());
        soundcard.validate(usb_controller)?;
        let (audio, audio_warnings) = self.display.audio(soundcard)?;
        warnings.extend(audio_warnings);

        let display = self.display.args(guest, arch)?;

        #[cfg(not(target_os = "macos"))]
        let spice = {
            if let DisplayType::Spice { .. } | DisplayType::SpiceApp { .. } = self.display.display_type {
                let public_dir_str = self.public_dir.as_ref().as_ref().map(|path| path.to_string_lossy());
                Some(self.display.spice_args(vm_name, guest, public_dir_str)?)
            } else {
                None
            }
        };

        let mouse = self.mouse.unwrap_or(guest.default_mouse());
        let usb = usb_controller.usb_args(guest, &self.usb_devices);

        Ok((
            IoArgs {
                display,
                audio,
                mouse,
                usb,
                keyboard,
                keyboard_layout: self.keyboard_layout,
                #[cfg(not(target_os = "macos"))]
                spice,
            },
            warnings,
        ))
    }
}

pub struct IoArgs<'a> {
    display: DisplayArgs,
    audio: Audio,
    mouse: Mouse,
    usb: USBArgs<'a>,
    keyboard: Keyboard,
    keyboard_layout: KeyboardLayout,
    #[cfg(not(target_os = "macos"))]
    spice: Option<spice::SpiceArgs<'a>>,
}

impl EmulatorArgs for IoArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let iter = chain!(
            self.display.display(),
            self.audio.display(),
            self.mouse.display(),
            self.usb.display(),
            self.keyboard.display(),
            self.keyboard_layout.display(),
        );

        #[cfg(not(target_os = "macos"))]
        let iter = iter.chain(self.spice.as_ref().map(|spice| spice.display()).into_iter().flatten());

        iter
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let iter = chain!(
            self.display.qemu_args(),
            self.audio.qemu_args(),
            self.mouse.qemu_args(),
            self.usb.qemu_args(),
            self.keyboard.qemu_args(),
            self.keyboard_layout.qemu_args(),
        );

        #[cfg(not(target_os = "macos"))]
        let iter = iter.chain(self.spice.as_ref().map(|spice| spice.qemu_args()).into_iter().flatten());

        iter
    }
}
