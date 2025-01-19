use std::{borrow::Cow, ffi::OsStr};

use crate::{
    data::{GuestOS, Mouse},
    utils::{EmulatorArgs, QemuArg},
};

impl GuestOS {
    pub(crate) fn default_mouse(&self) -> Mouse {
        match self {
            GuestOS::FreeBSD | GuestOS::GhostBSD => Mouse::Usb,
            _ => Mouse::Tablet,
        }
    }
}

impl EmulatorArgs for Mouse {
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let device = match self {
            Self::PS2 => return vec![],
            Self::Usb => "usb-mouse,bus=input.0",
            Self::Tablet => "usb-tablet,bus=input.0",
            Self::Virtio => "virtio-mouse",
        };
        vec![Cow::Borrowed(OsStr::new("-device")), Cow::Borrowed(OsStr::new(device))]
    }
}
