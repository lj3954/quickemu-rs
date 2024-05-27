use crate::config::{DiskFormat, DiskImage, GuestOS, Image, MacOSRelease, PreAlloc};
use crate::config_parse::BYTES_PER_GB;
use crate::qemu_args::external_command;
use anyhow::{anyhow, bail, Result};
use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

const MIN_DISK_SIZE: u64 = 197_632 * 8;

pub fn image_args(vm_dir: &Path, images: Option<Vec<Image>>, disks: Vec<DiskImage>, guest_os: &GuestOS, status_quo: bool) -> Result<(Vec<OsString>, Option<Vec<String>>)> {
    let qemu_img = which("qemu-img").map_err(|_| anyhow!("qemu-img could not be found. Please verify that QEMU is installed on your system."))?;

    let mut print_args = Vec::new();

    let disk_used = disks
        .iter()
        .map(|disk| {
            if !disk.path.exists() {
                let disk_format = disk.format.as_ref().unwrap();
                let size = disk.size.unwrap_or(guest_os.disk_size());
                external_command::create_disk_image(&qemu_img, &disk.path, size, disk_format, &disk.preallocation)?;
                print_args.push(format!(
                    "Created {} disk image {} with size {} GiB. Preallocation: {}",
                    disk_format.as_ref(),
                    disk.path.display(),
                    size as f64 / BYTES_PER_GB as f64,
                    disk.preallocation
                ));
                Ok(false)
            } else {
                let image_info = Command::new(&qemu_img)
                    .arg("info")
                    .arg(&disk.path)
                    .output()
                    .map_err(|e| anyhow!("Could not read disk image information using qemu-img: {}", e))?;
                if !image_info.status.success() {
                    bail!(
                        "Failed to get write lock on disk image {}. Please ensure that the disk image is not already in use.",
                        &disk.path.display()
                    );
                }
                let disk_size = String::from_utf8_lossy(&image_info.stdout)
                    .lines()
                    .find_map(|line| {
                        if line.starts_with("virtual size: ") {
                            Some(line.split_whitespace().skip(2).take(2).collect::<Vec<&str>>().join(" "))
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow!("Could not read disk size."))?;
                print_args.push(format!("Using disk image: {}. Size: {}", disk.path.display(), disk_size));
                Ok(disk.preallocation != PreAlloc::Off
                    || disk
                        .path
                        .metadata()
                        .map_err(|e| anyhow!("Could not read disk size: {}", e))?
                        .len()
                        > MIN_DISK_SIZE)
            }
        })
        .collect::<Result<Vec<bool>>>()?
        .into_iter()
        .any(|disk_used| disk_used);

    let mut used_indices: HashSet<u8> = HashSet::new();
    let mut args = disk_img_args(disks, guest_os, vm_dir, status_quo, &mut used_indices)?;

    if !disk_used {
        let (mut image_args, mut print) = image_file_args(images.unwrap(), guest_os, vm_dir, &mut used_indices);
        args.append(&mut image_args);
        print_args.append(&mut print);
    }

    Ok((args, Some(print_args)))
}

const MAC_BOOTLOADER: [&str; 2] = ["OpenCore.qcow2", "ESP.qcow2"];
// Ensure that the disk image is the final argument from this function. Status Quo argument is to
// be added afterwards
fn disk_img_args(disks: Vec<DiskImage>, guest_os: &GuestOS, vm_dir: &Path, status_quo: bool, used_indices: &mut HashSet<u8>) -> Result<Vec<OsString>> {
    let mut key = 1;
    let mut args: Vec<OsString> = match guest_os {
        GuestOS::MacOS { .. } => {
            let bootloader = MAC_BOOTLOADER
                .iter()
                .find_map(|bootloader| {
                    let bootloader = vm_dir.join(bootloader);
                    if bootloader.exists() {
                        Some(bootloader)
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow!("Could not find macOS bootloader. Please ensure that `OpenCore.qcow2` or `ESP.qcow2` is located within your VM directory."))?;
            vec![
                "-device".into(),
                "ahci,id=ahci".into(),
                "-device".into(),
                "ide-hd,bus=ahci.0,drive=BootLoader,bootindex=0".into(),
                "-drive".into(),
                disk_arg(bootloader, "BootLoader", "qcow2"),
            ]
        }
        GuestOS::KolibriOS => vec!["-device".into(), "ahci,id=ahci".into()],
        _ => Vec::new(),
    };

    for disk in disks {
        let disk_format = disk.format.unwrap();
        if !matches!(disk_format, DiskFormat::Qcow2 | DiskFormat::Raw) {
            log::warn!(
                "This project does not officially support disk format {}. Unintended behavior, including data corruption, may occur. Proceed with caution.",
                disk_format.as_ref()
            );
        }
        let disk_format = disk_format.as_ref();
        match guest_os {
            GuestOS::MacOS { release } if release >= &MacOSRelease::Catalina => args.extend(handle_disks(
                disk.path,
                "virtio-blk-pci,drive=",
                "SystemDisk",
                disk_format,
                &mut key,
            )),
            GuestOS::MacOS { .. } => args.extend(handle_disks(
                disk.path,
                "ide-hd,bus=ahci.2,drive=",
                "SystemDisk",
                disk_format,
                &mut key,
            )),
            GuestOS::KolibriOS => args.extend(handle_disks(
                disk.path,
                "ide-hd,bus=ahci.0,drive=",
                "SystemDisk",
                disk_format,
                &mut key,
            )),
            GuestOS::Batocera => args.extend(handle_disks(
                disk.path,
                "virtio-blk-pci,drive=",
                "SystemDisk",
                disk_format,
                &mut key,
            )),
            GuestOS::ReactOS => {
                // ReactOS ISO must always be mounted at index 2. Ensure that the index is skipped
                used_indices.insert(2);
                args.push("-drive".into());
                args.push(reactos_arg(disk.path, 0, "disk", used_indices));
                used_indices.remove(&2);
            }
            GuestOS::WindowsServer => args.extend(handle_disks(disk.path, "ide-hd,drive=", "SystemDisk", disk_format, &mut key)),
            _ => args.extend(handle_disks(
                disk.path,
                "virtio-blk-pci,drive=",
                "SystemDisk",
                disk_format,
                &mut key,
            )),
        }
        if status_quo {
            args.push("-snapshot".into());
        }
    }

    Ok(args)
}

fn handle_disks(disk: PathBuf, arg: &str, diskname: &str, disk_format: &str, key: &mut u8) -> [OsString; 4] {
    let diskname = match key {
        1 => diskname.into(),
        _ => "Disk".to_string() + &key.to_string(),
    };
    let mut arg = OsString::from(arg);
    arg.push(&diskname);
    *key += 1;
    ["-device".into(), arg, "-drive".into(), disk_arg(disk, &diskname, disk_format)]
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

fn image_file_args(images: Vec<Image>, guest_os: &GuestOS, vm_dir: &Path, used_indices: &mut HashSet<u8>) -> (Vec<OsString>, Vec<String>) {
    let print = images.iter().map(|image| image.to_string()).collect::<Vec<String>>();
    let args = images
        .into_iter()
        .flat_map(|image| match image {
            Image::Iso(iso) => match guest_os {
                GuestOS::FreeDOS => vec!["-boot".into(), "order=dc".into(), "-drive".into(), iso_arg(iso, 0, used_indices)],
                GuestOS::KolibriOS => vec!["drive".into(), iso_arg(iso, 2, used_indices)],
                GuestOS::ReactOS => vec!["-boot".into(), "order=d".into(), "-drive".into(), reactos_arg(iso, 2, "cdrom", used_indices)],
                GuestOS::Windows => {
                    let unattended = vm_dir.join("unattended.iso");
                    if unattended.exists() {
                        vec!["-drive".into(), iso_arg(iso, 0, used_indices), "-drive".into(), iso_arg(unattended, 2, used_indices)]
                    } else {
                        vec!["-drive".into(), iso_arg(iso, 0, used_indices)]
                    }
                }
                _ => vec!["-drive".into(), iso_arg(iso, 0, used_indices)],
            },
            Image::Img(img) => match guest_os {
                GuestOS::MacOS { .. } => vec!["-device".into(), "ide-hd,bus=ahci.1,drive=RecoveryImage".into(), "-drive".into(), img_arg(img, "RecoveryImage")],
                _ => vec!["-device".into(), "virtio-blk-pci,drive=BootDisk".into(), "-drive".into(), img_arg(img, "BootDisk")],
            },
            Image::FixedIso(iso) => vec!["-drive".into(), iso_arg(iso, 1, used_indices)],
            Image::Floppy(floppy) => vec!["-drive".into(), floppy_arg(floppy)],
        })
        .collect::<Vec<OsString>>();
    (args, print)
}

fn floppy_arg(floppy: PathBuf) -> OsString {
    let mut argument = OsString::from("if=floppy,format=raw,file=");
    argument.push(floppy);
    argument
}

fn reactos_arg(file: PathBuf, index: u8, media_type: &str, used_indices: &mut HashSet<u8>) -> OsString {
    let index = find_next_index(index, used_indices);
    let mut argument = OsString::from("if=ide,index=");
    argument.push(index);
    argument.push(",media=");
    argument.push(media_type);
    argument.push(",file=");
    argument.push(file);
    argument
}

fn iso_arg(iso: PathBuf, index: u8, used_indices: &mut HashSet<u8>) -> OsString {
    let index = find_next_index(index, used_indices);
    let mut argument = OsString::from("media=cdrom,index=");
    argument.push(index);
    argument.push(",file=");
    argument.push(iso);
    argument
}

fn img_arg(img: PathBuf, id: &str) -> OsString {
    let mut argument = OsString::from("id=");
    argument.push(id);
    argument.push(",if=none,format=raw,file=");
    argument.push(img);
    argument
}

fn find_next_index(index: u8, used_indices: &mut HashSet<u8>) -> String {
    (index..=u8::MAX)
        .find(|index| used_indices.insert(*index))
        .expect("Could not find next available index. You may have an unsupported amount of images (>255).")
        .to_string()
}
