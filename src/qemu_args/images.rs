use anyhow::{anyhow, bail, Result};
use std::ffi::OsString;
use which::which;
use crate::config::{Image, GuestOS, PreAlloc, MacOSRelease};
use crate::config_parse::BYTES_PER_GB;
use std::path::{PathBuf, Path};
use std::process::Command;

const MIN_DISK_SIZE: u64 = 197_632 * 8;

pub fn image_args(vm_dir: &Path, images: (Image, Option<PathBuf>, Option<PathBuf>), disk: PathBuf, disk_size: Option<u64>, guest_os: &GuestOS, preallocation: &PreAlloc, status_quo: bool) -> Result<(Vec<OsString>, Option<Vec<String>>)> {
    let (image, fixed_iso, floppy) = images;

    let qemu_img = which("qemu-img").map_err(|_| anyhow!("qemu-img could not be found. Please verify that QEMU is installed on your system."))?;
    let disk_size = disk_size.unwrap_or(guest_os.disk_size());
    let disk_format = disk_format(&disk.to_string_lossy(), preallocation)?;

    let mut print_args = Vec::new();

    let disk_used = if !disk.exists() {
        if image == Image::None {
            bail!("Disk image does not exist, and no image file was specified. Please provide an `img` or `iso` in your configuration file.");
        }
        Command::new(qemu_img)
            .arg("create")
            .arg("-q")
            .arg("-f")
            .arg(disk_format)
            .arg("-o")
            .arg(preallocation.qemu_arg())
            .arg(&disk)
            .arg(disk_size.to_string())
            .output()
            .map_err(|e| anyhow!("Could not create disk image: {}", e))?;
        print_args.push(format!("Created disk image {} with size {} GiB. Preallocation: {}", disk.to_string_lossy(), disk_size / BYTES_PER_GB, preallocation));
        false
    } else if guest_os == &GuestOS::KolibriOS {
        print_args.push(format!("Disk image: {}. Size: {} GiB", disk.to_string_lossy(), disk_size / BYTES_PER_GB));
        false
    } else {
        Command::new(qemu_img)
            .arg("info")
            .arg(&disk)
            .output()
            .map_err(|_| anyhow!("Failed to get write lock on disk image. Please ensure that the disk image is not already in use."))?;
        print_args.push(format!("Using disk image: {}. Size: {} GiB", disk.to_string_lossy(), disk_size / BYTES_PER_GB));
        preallocation != &PreAlloc::Off || disk.metadata().map_err(|e| anyhow!("Could not read disk size: {}", e))?.len() > MIN_DISK_SIZE
    };

    


    let mut args = disk_img_args(disk, disk_format, guest_os, vm_dir)?;
    if status_quo {
        args.push("-snapshot".into());
    }

    if !disk_used {
        print_args.push(image.to_string());
        let mut image_args = image.into_args(guest_os, vm_dir);
        if let Some(iso) = fixed_iso {
            print_args.push(format!("Fixed ISO (CD-ROM): {}", iso.to_string_lossy()));
            image_args.push("-drive".into());
            image_args.push(iso_arg(iso, "1"));
        }
        if let Some(floppy) = floppy {
            print_args.push(format!("Floppy: {}", floppy.to_string_lossy()));
            image_args.push("-drive".into());
            let mut floppy_arg = OsString::from("if=floppy,format=raw,file=");
            floppy_arg.push(floppy);
            image_args.push(floppy_arg);
        }
        args.append(&mut image_args);
    }

    Ok((args, Some(print_args)))
}

const MAC_BOOTLOADER: [&str; 2] = ["OpenCore.qcow2", "ESP.qcow2"];
// Ensure that the disk image is the final argument from this function. Status Quo argument is to
// be added afterwards
fn disk_img_args(disk: PathBuf, disk_format: &str, guest_os: &GuestOS, vm_dir: &Path) -> Result<Vec<OsString>> {
    Ok(match guest_os {
        GuestOS::MacOS(release) => {
            let bootloader = MAC_BOOTLOADER.iter().find_map(|bootloader| {
                let bootloader = vm_dir.join(bootloader);
                if bootloader.exists() {
                    Some(bootloader)
                } else {
                    None
                }
            }).ok_or_else(|| anyhow!("Could not find macOS bootloader. Please ensure that `OpenCore.qcow2` or `ESP.qcow2` is located within your VM directory."))?;

            let disk_device = if release >= &MacOSRelease::Catalina {
                OsString::from("virtio-blk-pci,drive=SystemDisk")
            } else {
                OsString::from("ide-hd,bus=ahci.2,drive=SystemDisk")
            };
            vec!["-device".into(), "ahci,id=ahci".into(), "-device".into(), "ide-hd,bus=ahci.0,drive=BootLoader,bootindex=0".into(), "-drive".into(), disk_arg(bootloader, "BootLoader", "qcow2"),
                "-device".into(), disk_device, "-drive".into(), disk_arg(disk, "SystemDisk", disk_format)]
        },
        GuestOS::KolibriOS => vec!["-device".into(), "ahci,id=ahci".into(), "-device".into(), "ide-hd,bus=ahci.0,drive=SystemDisk".into(), "-drive".into(), disk_arg(disk, "SystemDisk", disk_format)],
        GuestOS::Batocera => vec!["-device".into(), "virtio-blk-pci,drive=SystemDisk".into(), "-drive".into(), disk_arg(disk, "SystemDisk", disk_format)],
        GuestOS::ReactOS => vec!["-drive".into(), reactos_arg(disk, "0", "disk")],
        GuestOS::WindowsServer => vec!["-device".into(), "ide-hd,drive=SystemDisk".into(), "-drive".into(), disk_arg(disk, "SystemDisk", disk_format)],
        _ => vec!["-device".into(), "virtio-blk-pci,drive=SystemDisk".into(), "-drive".into(), disk_arg(disk, "SystemDisk", disk_format)],
    })
}

fn disk_arg(disk: PathBuf, id: &str, disk_format: &str) -> OsString {
    let mut argument = OsString::from("id=");
    argument.push(id);
    argument.push(",if=none,format=");
    argument.push(disk_format);
    argument.push(",file=");
    argument.push(disk);
    argument
}

impl Image {
    pub fn into_args(self, guest_os: &GuestOS, vm_dir: &Path) -> Vec<OsString> {
        match self {
            Self::None => vec![],
            Self::Iso(iso) => match guest_os {
                GuestOS::FreeDOS => vec!["-boot".into(), "order=dc".into(), "-drive".into(), iso_arg(iso, "0")],
                GuestOS::KolibriOS => vec!["drive".into(), iso_arg(iso, "2")],
                GuestOS::ReactOS => vec!["-boot".into(), "order=d".into(), "-drive".into(), reactos_arg(iso, "2", "cdrom")],
                GuestOS::Windows => {
                    let unattended = vm_dir.join("unattended.iso");
                    if unattended.exists() {
                        vec!["-drive".into(), iso_arg(iso, "0"), "-drive".into(), iso_arg(unattended, "2")]
                    } else {
                        vec!["-drive".into(), iso_arg(iso, "0")]
                    }
                },
                _ => vec!["-drive".into(), iso_arg(iso, "0")],
            },
            Self::Img(img) => match guest_os {
                GuestOS::MacOS(_) => vec!["-device".into(), "ide-hd,bus=ahci.1,drive=RecoveryImage".into(), "-drive".into(), img_arg(img, "RecoveryImage")],
                _ => vec!["-device".into(), "virtio-blk-pci,drive=BootDisk".into(), "-drive".into(), img_arg(img, "BootDisk")],
            }
        }
    }
}

fn reactos_arg(file: PathBuf, index: &str, media_type: &str) -> OsString {
    let mut argument = OsString::from("if=ide,index=");
    argument.push(index);
    argument.push(",media=");
    argument.push(media_type);
    argument.push(",file=");
    argument.push(file);
    argument
}

fn iso_arg(iso: PathBuf, index: &str) -> OsString {
    let mut argument = OsString::from("media=cdrom,index=");
    argument.push(index);
    argument.push(",file=");
    argument.push(iso);
    argument
}

fn img_arg(img: PathBuf, id: &str) -> OsString {
    let mut argument = OsString::from("-drive id=");
    argument.push(id);
    argument.push(",if=none,format=raw,file=");
    argument.push(img);
    argument
}


const UNSUPPORTED_FORMATS: [&str; 5] = ["qed", "qcow", "vdi", "vpc", "vhdx"];
fn disk_format(image_name: &str, prealloc: &PreAlloc) -> Result<&'static str> {
    Ok(match image_name.split('.').last().ok_or_else(|| anyhow!("Could not find disk image file extension."))? {
        "raw" if prealloc == &PreAlloc::Metadata => bail!("`raw` disk images do not support the metadata preallocation type."),
        "raw" => "raw",
        "qcow2" => "qcow2",
        _ if prealloc != &PreAlloc::Off => bail!("Only `raw` and `qcow2` disk images support preallocation."),
        image_type => match UNSUPPORTED_FORMATS.into_iter().find(|format| image_type == *format) {
            Some(format) => {
                log::warn!("This project does not officially support disk format {}. Unintended behavior, including data corruption, may occur. Proceed with caution.", format);
                format
            },
            None => bail!("Disk image format '{}' is not supported.", image_type),
        },
    })
}
