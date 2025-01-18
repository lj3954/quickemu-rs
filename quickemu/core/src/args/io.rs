#![allow(unused_lifetimes)]
use audio::Audio;
use display::DisplayArgs;
use itertools::chain;

use crate::{
    data::{Arch, DisplayType, GuestOS, Io},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

mod audio;
mod display;
#[cfg(not(target_os = "macos"))]
mod spice;

impl<'a> Io {
    pub fn args(&'a self, arch: Arch, guest: GuestOS, vm_name: &'a str) -> Result<(IoArgs<'a>, Vec<Warning>), Error> {
        let mut warnings = Vec::new();
        let (audio, audio_warnings) = self.display.audio(self.soundcard)?;
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

        Ok((
            IoArgs {
                display,
                audio,
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
    #[cfg(not(target_os = "macos"))]
    spice: Option<spice::SpiceArgs<'a>>,
}

impl EmulatorArgs for IoArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let iter = chain!(self.display.display(), self.audio.display(),);
        #[cfg(not(target_os = "macos"))]
        let iter = iter.chain(self.spice.as_ref().map(|spice| spice.display()).into_iter().flatten());
        iter
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let iter = chain!(self.display.qemu_args(), self.audio.qemu_args(),);
        #[cfg(not(target_os = "macos"))]
        let iter = iter.chain(self.spice.as_ref().map(|spice| spice.qemu_args()).into_iter().flatten());
        iter
    }
}
