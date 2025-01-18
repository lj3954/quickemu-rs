use crate::{
    data::{Display, DisplayType, SoundCard},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};
use std::{borrow::Cow, ffi::OsStr};

impl Display {
    pub(crate) fn audio(&self, sound_card: SoundCard) -> Result<(Audio, Option<Warning>), Error> {
        let backend = match sound_card {
            SoundCard::None => None,
            _ => match self.display_type {
                #[cfg(not(target_os = "macos"))]
                DisplayType::Spice { .. } | DisplayType::SpiceApp { .. } | DisplayType::None => Some(AudioBackend::Spice),
                #[cfg(target_os = "macos")]
                _ => Some(AudioBackend::CoreAudio),
                #[cfg(target_os = "linux")]
                _ => {
                    if process_active("pipewire") {
                        Some(AudioBackend::PipeWire)
                    } else if process_active("pulseaudio") {
                        Some(AudioBackend::PulseAudio)
                    } else {
                        return Err(Error::AudioBackend);
                    }
                }
                #[cfg(not(any(target_os = "linux", target_os = "macos")))]
                _ => None,
            },
        };
        Ok((Audio { sound_card, backend }, None))
    }
}

#[cfg(target_os = "linux")]
fn process_active(name: &str) -> bool {
    std::process::Command::new("pidof")
        .arg(name)
        .output()
        .is_ok_and(|o| o.status.success())
}

pub(crate) struct Audio {
    sound_card: SoundCard,
    backend: Option<AudioBackend>,
}

enum AudioBackend {
    #[cfg(not(target_os = "macos"))]
    Spice,
    #[cfg(target_os = "macos")]
    CoreAudio,
    #[cfg(target_os = "linux")]
    PipeWire,
    #[cfg(target_os = "linux")]
    PulseAudio,
}

impl EmulatorArgs for Audio {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let sound_type = match self.sound_card {
            SoundCard::None => "Disabled",
            SoundCard::AC97 => "AC97",
            SoundCard::ES1370 => "ES1370",
            SoundCard::SB16 => "Sound Blaster 16",
            SoundCard::IntelHDA => "Intel HDA",
        };
        Some(ArgDisplay {
            name: "Sound".into(),
            value: sound_type.into(),
        })
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut args = Vec::new();
        if let Some(backend) = &self.backend {
            args.push(Cow::Borrowed(OsStr::new("-audiodev")));
            args.push(Cow::Borrowed(OsStr::new(match backend {
                #[cfg(not(target_os = "macos"))]
                AudioBackend::Spice => "spice,id=audio0",
                #[cfg(target_os = "macos")]
                AudioBackend::CoreAudio => "coreaudio,id=audio0",
                #[cfg(target_os = "linux")]
                AudioBackend::PipeWire => "pipewire,id=audio0",
                #[cfg(target_os = "linux")]
                AudioBackend::PulseAudio => "pulse,id=audio0",
            })));
        }
        match self.sound_card {
            SoundCard::None => {}
            SoundCard::AC97 => args.extend([Cow::Borrowed(OsStr::new("-device")), Cow::Borrowed(OsStr::new("ac97,audiodev=audio0"))]),
            SoundCard::ES1370 => args.extend([Cow::Borrowed(OsStr::new("-device")), Cow::Borrowed(OsStr::new("es1370,audiodev=audio0"))]),
            SoundCard::SB16 => args.extend([Cow::Borrowed(OsStr::new("-device")), Cow::Borrowed(OsStr::new("sb16,audiodev=audio0"))]),
            SoundCard::IntelHDA => args.extend([
                Cow::Borrowed(OsStr::new("-device")),
                Cow::Borrowed(OsStr::new("intel-hda")),
                Cow::Borrowed(OsStr::new("-device")),
                Cow::Borrowed(OsStr::new("hda-duplex,audiodev=audio0")),
            ]),
        }

        args
    }
}
