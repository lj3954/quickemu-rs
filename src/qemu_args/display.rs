use anyhow::{anyhow, bail, Result};
use crate::config::{Display, SoundCard};

impl SoundCard {
    pub fn to_args(&self) -> (Vec<String>, Option<Vec<String>>) {
        match self {
            Self::None => (vec![], Some(vec!["Sound: Disabled".into()])),
            Self::AC97 => (vec!["-device".into(), "ac97,audiodev=audio0".into()], Some(vec!["Emulated sound card: AC97".into()])),
            Self::ES1370 => (vec!["-device".into(), "es1370,audiodev=audio0".into()], Some(vec!["Emulated sound card: ES1370".into()])),
            Self::SB16 => (vec!["-device".into(), "sb16,audiodev=audio0".into()], Some(vec!["Emulated sound card: Sound Blaster 16".into()])),
            Self::IntelHDA => (vec!["-device".into(), "intel-hda".into(), "-device".into(), "hda-duplex,audiodev=audio0".into()], Some(vec!["Emulated sound card: Intel HDA".into()])),
        }
    }
}

