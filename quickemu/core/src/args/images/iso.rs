use std::{borrow::Cow, collections::HashSet, ffi::OsString, path::Path};

use crate::{
    arg,
    data::{GuestOS, Images},
    error::Error,
    oarg,
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

impl<'a> Images {
    pub(crate) fn iso_args(&'a self, installed: bool, guest: GuestOS, vm_dir: &Path, used_indices: &mut HashSet<u32>) -> Result<IsoArgs<'a>, Error> {
        let mut key = 0;
        let mut images = Vec::new();

        if let GuestOS::Windows = guest {
            let unattended: Cow<'a, Path> = Cow::Owned(vm_dir.join("unattended.iso"));
            if unattended.exists() {
                images.push(MountedIso::new(unattended, &mut 2, used_indices));
            }
        }

        let images = self
            .iso
            .iter()
            .filter(|iso| iso.always_mount || !installed)
            .map(|iso| {
                if iso.path.is_absolute() {
                    Cow::Borrowed(iso.path.as_path())
                } else {
                    Cow::Owned(vm_dir.join(&iso.path))
                }
            })
            .try_fold(images, |mut acc, path| {
                if !path.exists() {
                    return Err(Error::NonexistentImage(path.display().to_string()));
                }
                acc.push(MountedIso::new(path, &mut key, used_indices));
                Ok(acc)
            })?;

        Ok(IsoArgs { images, guest })
    }
}

pub(crate) struct IsoArgs<'a> {
    images: Vec<MountedIso<'a>>,
    guest: GuestOS,
}

impl EmulatorArgs for IsoArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        self.images.iter().map(|iso| ArgDisplay {
            name: Cow::Borrowed("ISO"),
            value: Cow::Owned(iso.path.display().to_string()),
        })
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut args = Vec::new();
        match self.guest {
            GuestOS::FreeDOS => args.extend([arg!("-boot"), arg!("order=dc")]),
            GuestOS::ReactOS => args.extend([arg!("-boot"), arg!("order=d")]),
            _ => {}
        }
        args.into_iter().chain(self.images.iter().flat_map(|iso| iso.args(self.guest)))
    }
}

struct MountedIso<'a> {
    path: Cow<'a, Path>,
    index: u32,
}

impl<'a> MountedIso<'a> {
    fn new(path: Cow<'a, Path>, key: &mut u32, used_indices: &mut HashSet<u32>) -> Self {
        while !used_indices.insert(*key) {
            *key += 1;
        }
        let mounted_iso = Self { path, index: *key };
        *key += 1;
        mounted_iso
    }

    fn args(&self, guest: GuestOS) -> [QemuArg; 2] {
        let arg = match guest {
            GuestOS::ReactOS => self.reactos_arg(),
            _ => {
                let mut arg = OsString::from("media=cdrom,index=");
                arg.push(self.index.to_string());
                arg.push(",file=");
                arg.push(self.path.as_ref());
                oarg!(arg)
            }
        };
        [arg!("-drive"), arg]
    }

    fn reactos_arg(&self) -> QemuArg {
        let mut arg = OsString::from("if=ide,index=");
        arg.push(self.index.to_string());
        arg.push(",media=cdrom,file=");
        arg.push(self.path.as_ref());
        oarg!(arg)
    }
}
