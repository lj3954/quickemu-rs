use std::{borrow::Cow, collections::HashSet, ffi::OsString, path::Path, process::Command};

use serde::Deserialize;
use size::Size;

use crate::{
    arg,
    data::{DiskFormat, GuestOS, Images, MacOSRelease, PreAlloc},
    error::Error,
    oarg,
    utils::{ArgDisplay, EmulatorArgs, QemuArg},
};

const MIN_DISK_SIZE: u64 = 197_632 * 8;
const MAC_BOOTLOADER: [&str; 2] = ["OpenCore.qcow2", "ESP.qcow2"];

impl<'a> Images {
    pub(crate) fn disk_args(&'a self, guest: GuestOS, vm_dir: &Path, status_quo: bool, used_indices: &mut HashSet<u32>) -> Result<DiskArgs<'a>, Error> {
        let mut key = 1;

        let non_disk_keys = match guest {
            // ReactOS ISO & Unattended windows installer must be mounted at index 2
            GuestOS::ReactOS | GuestOS::Windows => vec![2],
            _ => Vec::new(),
        };

        let bootloader = matches!(guest, GuestOS::MacOS { .. })
            .then(|| {
                MAC_BOOTLOADER
                    .iter()
                    .map(|p| vm_dir.join(p))
                    .find(|p| p.exists())
                    .ok_or(Error::MacBootloader)
                    .map(|p| MountedDisk {
                        path: Cow::Owned(p),
                        format: DiskFormat::Qcow2 { preallocation: PreAlloc::Off },
                        index: 0,
                        is_new: false,
                        size: 0,
                    })
            })
            .transpose()?;
        let ahci = matches!(guest, GuestOS::MacOS { .. } | GuestOS::KolibriOS);

        non_disk_keys.iter().for_each(|key| {
            used_indices.insert(*key);
        });

        let mut installed = false;

        let mounted_disks = self
            .disk
            .iter()
            .map(|disk| {
                let path = if disk.path.is_absolute() {
                    Cow::Borrowed(disk.path.as_path())
                } else {
                    Cow::Owned(vm_dir.join(&disk.path))
                };
                Ok(if !path.exists() {
                    let size = disk.size.unwrap_or(guest.default_disk_size());
                    create_disk_image(&path, size, disk.format)?;
                    MountedDisk::new(path, disk.format, &mut key, used_indices, true, size)
                } else {
                    let QemuImgInfo { actual_size, virtual_size } = find_disk_size(&path)?;
                    if disk.format.prealloc_enabled() || actual_size >= MIN_DISK_SIZE {
                        installed = true;
                    }
                    MountedDisk::new(path, disk.format, &mut key, used_indices, false, virtual_size)
                })
            })
            .collect::<Result<Vec<MountedDisk<'a>>, Error>>()?;

        non_disk_keys.iter().for_each(|key| {
            used_indices.remove(key);
        });

        Ok(DiskArgs {
            guest,
            mounted_disks,
            status_quo,
            ahci,
            bootloader,
            installed,
        })
    }
}

fn create_disk_image(path: &Path, size: u64, format: DiskFormat) -> Result<(), Error> {
    #[cfg(not(feature = "inbuilt_commands"))]
    let mut command = Command::new("qemu-img");

    command
        .arg("create")
        .arg("-q")
        .arg("-f")
        .arg(format.as_ref())
        .arg(path)
        .arg(size.to_string())
        .arg("-o")
        .arg(format.prealloc_arg());

    let output = command.output().map_err(|e| Error::Command("qemu-img", e.to_string()))?;

    if !output.status.success() {
        return Err(Error::DiskCreationFailed(String::from_utf8_lossy(&output.stderr).to_string()));
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct QemuImgInfo {
    actual_size: u64,
    virtual_size: u64,
}

fn find_disk_size(path: &Path) -> Result<QemuImgInfo, Error> {
    #[cfg(not(feature = "inbuilt_commands"))]
    let mut command = Command::new("qemu-img");

    command.arg("info").arg(path).arg("--output=json");

    let output = command.output().map_err(|e| Error::Command("qemu-img", e.to_string()))?;

    if !output.status.success() {
        return Err(Error::DiskInUse(path.display().to_string()));
    }

    serde_json::from_slice(&output.stdout).map_err(|e| Error::DeserializeQemuImgInfo(e.to_string()))
}

pub(crate) struct DiskArgs<'a> {
    guest: GuestOS,
    mounted_disks: Vec<MountedDisk<'a>>,
    status_quo: bool,
    ahci: bool,
    bootloader: Option<MountedDisk<'a>>,
    installed: bool,
}

impl DiskArgs<'_> {
    pub(crate) fn installed(&self) -> bool {
        self.installed
    }
}

impl EmulatorArgs for DiskArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        self.mounted_disks.iter().map(MountedDisk::arg_display)
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut args = Vec::new();
        if self.ahci {
            args.extend([arg!("-device"), arg!("ahci,id=ahci")]);
        }
        if let Some(bootloader) = &self.bootloader {
            args.extend(bootloader.args(self.guest));
            if self.status_quo {
                args.push(arg!("-snapshot"));
            }
        }

        args.into_iter().chain(self.mounted_disks.iter().flat_map(|disk| {
            let mut args = disk.args(self.guest);
            if self.status_quo {
                args.push(arg!("-snapshot"));
            }
            args
        }))
    }
}

struct MountedDisk<'a> {
    path: Cow<'a, Path>,
    format: DiskFormat,
    index: u32,
    is_new: bool,
    size: u64,
}

impl<'a> MountedDisk<'a> {
    fn args(&self, guest: GuestOS) -> Vec<QemuArg> {
        let (bus_arg, disk_name) = match guest {
            GuestOS::MacOS { release } if release < MacOSRelease::Catalina => ("ide-hd,bus=ahci.2,drive=", "SystemDisk"),
            GuestOS::KolibriOS => ("ide-hd,bus=ahci.0,drive=", "SystemDisk"),
            GuestOS::ReactOS => return self.reactos_args(),
            _ => ("virtio-blk-pci,drive=", "SystemDisk"),
        };

        let disk_name = match self.index {
            0 => "Bootloader",
            1 => disk_name,
            _ => &format!("Disk{}", self.index),
        };
        let device = bus_arg.to_string() + disk_name;

        let mut drive_arg = OsString::from("id=");
        drive_arg.push(disk_name);
        drive_arg.push(",if=none,format=");
        drive_arg.push(self.format.as_ref());
        drive_arg.push(",file=");
        drive_arg.push(self.path.as_ref());

        vec![arg!("-device"), oarg!(device), arg!("-drive"), oarg!(drive_arg)]
    }

    fn reactos_args(&self) -> Vec<QemuArg> {
        let mut argument = OsString::from("if=ide,index=");
        argument.push(self.index.to_string());
        argument.push(",media=disk,file=");
        argument.push(self.path.as_ref());
        vec![arg!("-drive"), oarg!(argument)]
    }

    fn new(path: Cow<'a, Path>, format: DiskFormat, key: &mut u32, used_indices: &mut HashSet<u32>, is_new: bool, size: u64) -> Self {
        while !used_indices.insert(*key) {
            *key += 1;
        }
        let disk = MountedDisk {
            path,
            format,
            index: *key,
            is_new,
            size,
        };
        *key += 1;
        disk
    }

    fn arg_display(&self) -> ArgDisplay {
        let name = if self.is_new { "Disk (Created)" } else { "Disk" };
        ArgDisplay {
            name: Cow::Borrowed(name),
            value: Cow::Owned(format!("{} ({})", self.path.display(), Size::from_bytes(self.size))),
        }
    }
}

impl GuestOS {
    fn default_disk_size(&self) -> u64 {
        let gib = match self {
            Self::Windows | Self::WindowsServer => 64,
            Self::MacOS { .. } => 96,
            Self::ReactOS | Self::KolibriOS => 16,
            _ => 32,
        };
        gib * size::consts::GiB as u64
    }
}
