use std::{borrow::Cow, ffi::OsString, path::Path};

use crate::{
    arg,
    data::{GuestOS, Images},
    error::Error,
    oarg,
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

impl<'a> Images {
    pub(crate) fn img_args(&'a self, installed: bool, vm_dir: &Path, guest: GuestOS) -> Result<ImgArgs<'a>, Error> {
        let images = self
            .img
            .iter()
            .filter(|img| img.always_mount || !installed)
            .map(|img| {
                if img.path.is_absolute() {
                    Cow::Borrowed(img.path.as_path())
                } else {
                    Cow::Owned(vm_dir.join(&img.path))
                }
            })
            .map(|img| {
                if !img.exists() {
                    return Err(Error::NonexistentImage(img.display().to_string()));
                }
                Ok(img)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ImgArgs { images, guest })
    }
}

pub(crate) struct ImgArgs<'a> {
    images: Vec<Cow<'a, Path>>,
    guest: GuestOS,
}

impl EmulatorArgs for ImgArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        self.images.iter().map(|img| ArgDisplay {
            name: Cow::Borrowed("IMG"),
            value: Cow::Owned(img.display().to_string()),
        })
    }

    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let first = self.images.first().map(|first| {
            let first_id = match self.guest {
                GuestOS::MacOS { .. } => "RecoveryImage",
                _ => "BootDisk",
            };
            img_args(first, first_id, self.guest)
        });

        let rest = self
            .images
            .iter()
            .skip(1)
            .enumerate()
            .flat_map(|(i, img)| img_args(img, &format!("Image{i}"), self.guest));

        std::iter::once(first).flatten().flatten().chain(rest)
    }
}

fn img_args(img: &Path, id: &str, guest: GuestOS) -> [QemuArg; 4] {
    let mut drive_arg = OsString::from("id=");
    drive_arg.push(id);
    drive_arg.push(",if=none,format=raw,file=");
    drive_arg.push(img);

    let mut device_arg = OsString::from(match guest {
        GuestOS::MacOS { .. } => "ide-hd,bus=ahci.1,drive=",
        _ => "virtio-blk-pci,drive=",
    });
    device_arg.push(id);

    [arg!("-device"), oarg!(device_arg), arg!("-drive"), oarg!(drive_arg)]
}
