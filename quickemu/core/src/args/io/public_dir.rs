use std::{
    borrow::Cow,
    ffi::{OsStr, OsString},
    path::Path,
};

use crate::{
    arg,
    data::GuestOS,
    oarg,
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

pub(crate) struct PublicDirArgs<'a> {
    path: &'a Path,
    guest: GuestOS,
}

impl<'a> PublicDirArgs<'a> {
    pub(crate) fn new(path: &'a Path, guest: GuestOS) -> Self {
        Self { path, guest }
    }
}

impl EmulatorArgs for PublicDirArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let mut display = Vec::new();
        if let GuestOS::MacOS { .. } = self.guest {
            display.push(ArgDisplay {
                name: Cow::Borrowed("WebDAV (Guest)"),
                value: Cow::Borrowed("first, build spice-webdavd (https://gitlab.gnome.org/GNOME/phodav/-/merge_requests/24)"),
            });
        }
        display.push(ArgDisplay {
            name: Cow::Borrowed("WebDAV (Guest)"),
            value: Cow::Borrowed("dav://localhost:9843/"),
        });

        match self.guest {
            GuestOS::MacOS { .. } => {
                if self.path.metadata().is_ok_and(|m| m.permissions().readonly()) {
                    display.push(ArgDisplay {
                        name: Cow::Borrowed("9P (Host)"),
                        value: Cow::Owned(format!("`sudo chmod -r 777 {}`", self.path.display())),
                    });
                }
                display.push(ArgDisplay {
                    name: Cow::Borrowed("9P (Guest)"),
                    value: Cow::Borrowed("sudo mount_9p Public-quickemu ~/Public"),
                });
            }
            GuestOS::Linux | GuestOS::LinuxOld => {
                display.push(ArgDisplay {
                    name: Cow::Borrowed("9P (Guest)"),
                    value: Cow::Borrowed("`sudo mount -t 9p -o trans=virtio,version=9p2000.L,msize=104857600 Public-quickemu ~/Public`"),
                });
            }
            _ => (),
        }

        display
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        if let GuestOS::Windows | GuestOS::WindowsServer = self.guest {
            return vec![];
        }
        let mut fs = OsString::from("local,id=fsdev0,path=");
        fs.push(self.path);
        fs.push(",security_model=mapped-xattr");

        let device = OsStr::new("virtio-9p-pci,fsdev=fsdev0,mount_tag=Public-quickemu");
        vec![arg!("-fsdev"), oarg!(fs), arg!("-device"), arg!(device)]
    }
}
