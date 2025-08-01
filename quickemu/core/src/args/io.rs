use audio::Audio;
use display::DisplayArgs;
use itertools::chain;
use public_dir::PublicDirArgs;
use usb::USBArgs;

use crate::{
    data::{Arch, GuestOS, Io, Keyboard, KeyboardLayout, Mouse},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, LaunchFn, QemuArg},
};

mod audio;
mod display;
mod keyboard;
mod mouse;
mod public_dir;
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

        let public_dir_args = self.public_dir.as_ref().as_deref().map(|d| PublicDirArgs::new(d, guest));

        #[cfg(not(target_os = "macos"))]
        let spice = matches!(self.display.display_type, DisplayType::Spice { .. } | DisplayType::SpiceApp)
            .then(|| {
                let public_dir_str = self.public_dir.as_ref().as_ref().map(|path| path.to_string_lossy());
                self.display.spice_args(vm_name, guest, public_dir_str)
            })
            .transpose()?;

        let mouse = self.mouse.unwrap_or(guest.default_mouse());
        let usb = usb_controller.usb_args(guest);

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
                public_dir_args,
            },
            warnings,
        ))
    }
}

pub struct IoArgs<'a> {
    display: DisplayArgs,
    audio: Audio,
    mouse: Mouse,
    usb: USBArgs,
    keyboard: Keyboard,
    keyboard_layout: KeyboardLayout,
    #[cfg(not(target_os = "macos"))]
    spice: Option<spice::SpiceArgs<'a>>,
    public_dir_args: Option<PublicDirArgs<'a>>,
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
            self.public_dir_args.as_ref().map(|d| d.display()).into_iter().flatten(),
        );

        #[cfg(not(target_os = "macos"))]
        let iter = iter.chain(self.spice.as_ref().map(|spice| spice.display()).into_iter().flatten());

        iter
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let iter = chain!(
            self.usb.qemu_args(),
            self.display.qemu_args(),
            self.audio.qemu_args(),
            self.mouse.qemu_args(),
            self.keyboard.qemu_args(),
            self.keyboard_layout.qemu_args(),
            self.public_dir_args.as_ref().map(|d| d.qemu_args()).into_iter().flatten(),
        );

        #[cfg(not(target_os = "macos"))]
        let iter = iter.chain(self.spice.as_ref().map(|spice| spice.qemu_args()).into_iter().flatten());

        iter
    }
    fn launch_fns(self) -> impl IntoIterator<Item = LaunchFn> {
        let iter = chain!(
            self.display.launch_fns(),
            self.audio.launch_fns(),
            self.mouse.launch_fns(),
            self.usb.launch_fns(),
            self.keyboard.launch_fns(),
            self.keyboard_layout.launch_fns(),
            self.public_dir_args.map(|d| d.launch_fns()).into_iter().flatten(),
        );

        #[cfg(not(target_os = "macos"))]
        let iter = iter.chain(self.spice.map(|spice| spice.launch_fns()).into_iter().flatten());

        iter
    }
}
