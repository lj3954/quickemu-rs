use crate::{
    arg,
    data::{Display, GuestOS, MacOSRelease, SoundCard, USBController},
    error::{Error, Warning},
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

#[cfg(target_os = "linux")]
use crate::data::DisplayType;

impl GuestOS {
    pub(crate) fn default_soundcard(&self) -> SoundCard {
        match self {
            GuestOS::FreeDOS => SoundCard::SB16,
            GuestOS::Solaris => SoundCard::AC97,
            GuestOS::MacOS { release } if release >= &MacOSRelease::BigSur => SoundCard::USBAudio,
            _ => SoundCard::IntelHDA,
        }
    }
}

impl SoundCard {
    pub(crate) fn validate(&self, usb_controller: USBController) -> Result<(), Error> {
        if matches!(self, SoundCard::USBAudio) && usb_controller != USBController::Xhci {
            return Err(Error::ConflictingSoundUsb);
        }
        Ok(())
    }
}

impl Display {
    pub(crate) fn audio(&self, sound_card: SoundCard) -> Result<(Audio, Option<Warning>), Error> {
        let backend = match sound_card {
            SoundCard::None => AudioBackend::None,
            _ => match self.display_type {
                #[cfg(not(target_os = "macos"))]
                DisplayType::Spice { .. } | DisplayType::SpiceApp | DisplayType::None => AudioBackend::Spice,
                #[cfg(target_os = "macos")]
                _ => AudioBackend::CoreAudio,
                #[cfg(target_os = "windows")]
                _ => AudioBackend::DirectSound,
                #[cfg(target_os = "linux")]
                _ => {
                    if process_active("pipewire") {
                        #[cfg(not(feature = "qemu_8_1"))]
                        {
                            AudioBackend::PulseAudio
                        }
                        #[cfg(feature = "qemu_8_1")]
                        AudioBackend::PipeWire
                    } else if process_active("pulseaudio") {
                        AudioBackend::PulseAudio
                    } else {
                        AudioBackend::Alsa
                    }
                }
                #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
                _ => AudioBackend::None,
            },
        };
        Ok((Audio { sound_card, backend }, None))
    }
}

#[cfg(target_os = "linux")]
fn process_active(name: &str) -> bool {
    let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::new()));
    let process = system.processes_by_exact_name(name).next();
    process.is_some()
}

pub(crate) struct Audio {
    sound_card: SoundCard,
    backend: AudioBackend,
}

enum AudioBackend {
    None,
    #[cfg(not(target_os = "macos"))]
    Spice,
    #[cfg(target_os = "macos")]
    CoreAudio,
    #[cfg(all(target_os = "linux", feature = "qemu_8_1"))]
    PipeWire,
    #[cfg(target_os = "linux")]
    PulseAudio,
    #[cfg(target_os = "linux")]
    Alsa,
    #[cfg(target_os = "windows")]
    DirectSound,
}

impl EmulatorArgs for Audio {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let sound_type = match self.sound_card {
            SoundCard::None => "Disabled",
            SoundCard::AC97 => "AC97",
            SoundCard::ES1370 => "ES1370",
            SoundCard::SB16 => "Sound Blaster 16",
            SoundCard::IntelHDA => "Intel HDA",
            SoundCard::USBAudio => "USB Audio",
        };
        Some(ArgDisplay {
            name: "Sound".into(),
            value: sound_type.into(),
        })
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let backend = match self.backend {
            AudioBackend::None => "none,id=audio0",
            #[cfg(not(target_os = "macos"))]
            AudioBackend::Spice => "spice,id=audio0",
            #[cfg(target_os = "macos")]
            AudioBackend::CoreAudio => "coreaudio,id=audio0",
            #[cfg(all(target_os = "linux", feature = "qemu_8_1"))]
            AudioBackend::PipeWire => "pipewire,id=audio0",
            #[cfg(target_os = "linux")]
            AudioBackend::PulseAudio => "pa,id=audio0",
            #[cfg(target_os = "linux")]
            AudioBackend::Alsa => "alsa,id=audio0",
            #[cfg(target_os = "windows")]
            AudioBackend::DirectSound => "dsound,id=audio0",
        };

        let mut args = vec![arg!("-audiodev"), arg!(backend)];

        match self.sound_card {
            SoundCard::None => {}
            SoundCard::AC97 => args.extend([arg!("-device"), arg!("ac97,audiodev=audio0")]),
            SoundCard::ES1370 => args.extend([arg!("-device"), arg!("es1370,audiodev=audio0")]),
            SoundCard::SB16 => args.extend([arg!("-device"), arg!("sb16,audiodev=audio0")]),
            SoundCard::USBAudio => args.extend([arg!("-device"), arg!("usb-audio,audiodev=audio0")]),
            SoundCard::IntelHDA => args.extend([arg!("-device"), arg!("intel-hda"), arg!("-device"), arg!("hda-duplex,audiodev=audio0")]),
        }

        args
    }
}
