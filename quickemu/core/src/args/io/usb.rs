use std::{borrow::Cow, ffi::OsStr};

use crate::{
    data::{GuestOS, MacOSRelease, USBController},
    utils::{EmulatorArgs, QemuArg},
};

impl GuestOS {
    pub(crate) fn default_usb_controller(&self) -> USBController {
        match self {
            GuestOS::Solaris => USBController::Xhci,
            GuestOS::MacOS { release } if release >= &MacOSRelease::BigSur => USBController::Xhci,
            _ => USBController::Ehci,
        }
    }
}

impl USBController {
    pub(crate) fn usb_args(&self, guest: GuestOS) -> USBArgs {
        #[cfg(not(target_os = "macos"))]
        let passthrough_controller = match self {
            Self::Ehci => Some(PassthroughController::UsbEhci),
            Self::Xhci => match guest {
                GuestOS::MacOS { release } if release >= MacOSRelease::BigSur => Some(PassthroughController::NecUsbXhci),
                _ => Some(PassthroughController::QemuXhci),
            },
            Self::None => None,
        };
        USBArgs {
            controller: *self,
            #[cfg(not(target_os = "macos"))]
            passthrough_controller,
        }
    }
}

pub(crate) struct USBArgs {
    controller: USBController,
    #[cfg(not(target_os = "macos"))]
    passthrough_controller: Option<PassthroughController>,
}

enum PassthroughController {
    NecUsbXhci,
    UsbEhci,
    QemuXhci,
}

impl PassthroughController {
    fn spice_arg(&self) -> &'static str {
        match self {
            Self::NecUsbXhci => "nec-usb-xhci,id=spicepass",
            Self::UsbEhci => "usb-ehci,id=spicepass",
            Self::QemuXhci => "qemu-xhci,id=spicepass",
        }
    }
}

impl EmulatorArgs for USBArgs {
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut args = vec![
            Cow::Borrowed(OsStr::new("-device")),
            Cow::Borrowed(OsStr::new("virtio-rng-pci")),
            Cow::Borrowed(OsStr::new("-object")),
            Cow::Borrowed(OsStr::new("rng-random,id=rng0,filename=/dev/urandom")),
        ];

        #[cfg(not(target_os = "macos"))]
        if let Some(passthrough_controller) = &self.passthrough_controller {
            args.extend([
                Cow::Borrowed(OsStr::new("-device")),
                Cow::Borrowed(OsStr::new(passthrough_controller.spice_arg())),
                Cow::Borrowed(OsStr::new("-chardev")),
                Cow::Borrowed(OsStr::new("spicevmc,id=usbredirchardev1,name=usbredir")),
                Cow::Borrowed(OsStr::new("-device")),
                Cow::Borrowed(OsStr::new("usb-redir,chardev=usbredirchardev1,id=usbredirdev1")),
                Cow::Borrowed(OsStr::new("-chardev")),
                Cow::Borrowed(OsStr::new("spicevmc,id=usbredirchardev2,name=usbredir")),
                Cow::Borrowed(OsStr::new("-device")),
                Cow::Borrowed(OsStr::new("usb-redir,chardev=usbredirchardev2,id=usbredirdev2")),
                Cow::Borrowed(OsStr::new("-chardev")),
                Cow::Borrowed(OsStr::new("spicevmc,id=usbredirchardev3,name=usbredir")),
                Cow::Borrowed(OsStr::new("-device")),
                Cow::Borrowed(OsStr::new("usb-redir,chardev=usbredirchardev3,id=usbredirdev3")),
                Cow::Borrowed(OsStr::new("-device")),
                Cow::Borrowed(OsStr::new("pci-ohci,id=smartpass")),
                Cow::Borrowed(OsStr::new("-device")),
                Cow::Borrowed(OsStr::new("usb-ccid")),
            ]);
        }

        match self.controller {
            USBController::Ehci => args.extend([Cow::Borrowed(OsStr::new("-device")), Cow::Borrowed(OsStr::new("usb-ehci,id=input"))]),
            USBController::Xhci => args.extend([Cow::Borrowed(OsStr::new("-device")), Cow::Borrowed(OsStr::new("qemu-xhci,id=input"))]),
            _ => (),
        }

        #[cfg(feature = "smartcard_args")]
        args.extend([
            Cow::Borrowed(OsStr::new("-chardev")),
            Cow::Borrowed(OsStr::new("spicevmc,id=ccid,name=smartcard")),
            Cow::Borrowed(OsStr::new("-device")),
            Cow::Borrowed(OsStr::new("ccid-card-passthru,chardev=ccid")),
        ]);

        args
    }
}
