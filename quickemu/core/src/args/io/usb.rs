use std::{borrow::Cow, ffi::OsStr};

use crate::{
    data::{GuestOS, MacOSRelease, USBController, USBDevices},
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

impl<'a> USBController {
    pub(crate) fn usb_args(&self, guest: GuestOS, devices: &'a USBDevices) -> USBArgs<'a> {
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
            passthrough_controller,
            usb_devices: devices,
        }
    }
}

pub(crate) struct USBArgs<'a> {
    controller: USBController,
    passthrough_controller: Option<PassthroughController>,
    usb_devices: &'a USBDevices,
}

enum PassthroughController {
    NecUsbXhci,
    UsbEhci,
    QemuXhci,
}

impl AsRef<str> for PassthroughController {
    fn as_ref(&self) -> &str {
        match self {
            Self::NecUsbXhci => "nec-usb-xhci",
            Self::UsbEhci => "usb-ehci",
            Self::QemuXhci => "qemu-xhci",
        }
    }
}

impl EmulatorArgs for USBArgs<'_> {
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
                Cow::Owned(format!("{},id=spicepass", passthrough_controller.as_ref()).into()),
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

        if self.usb_devices.as_ref().is_some() {
            todo!("USB device passthrough");
        }

        args
    }
}
