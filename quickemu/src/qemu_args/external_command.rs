#[cfg(not(target_os = "macos"))]
use crate::config::Viewer;
use crate::config::{DiskFormat, Display, PreAlloc};
use anyhow::{anyhow, bail, Result};
use std::{
    ffi::OsString,
    fs::File,
    path::Path,
    process::{Child, Command, Stdio},
};

pub fn launch_qemu(qemu_bin: &Path, args: &[OsString], display: &Display) -> Result<()> {
    match display {
        Display::Sdl => Command::new(qemu_bin)
            .args(args)
            .env("SDL_MOUSE_FOCUS_CLICKTHROUGH", "1")
            .spawn(),
        _ => Command::new(qemu_bin).args(args).spawn(),
    }
    .map_err(|e| anyhow!("Failed to start QEMU: {}", e))?;
    Ok(())
}

#[cfg(feature = "get_qemu_ver")]
pub fn qemu_version_process(qemu_bin: &Path) -> Result<Child> {
    Command::new(qemu_bin)
        .arg("--version")
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|e| anyhow!("Failed to find {} version: {}", qemu_bin.display(), e))
}

pub fn smartcard_process(qemu_bin: &Path) -> Result<Child> {
    Ok(Command::new(qemu_bin)
        .arg("-device")
        .arg("help")
        .stdout(Stdio::piped())
        .spawn()?)
}

pub fn tpm_pid(swtpm: &Path, args: &[OsString], log: File) -> Result<u32> {
    Ok(Command::new(swtpm)
        .args(args)
        .stderr(log)
        .spawn()
        .map_err(|_| anyhow!("Failed to start swtpm. Please check the log file in your VM directory for more information."))?
        .id())
}

#[cfg(not(target_os = "macos"))]
pub fn launch_viewer(viewer: &Path, vm_name: &str, publicdir: &str, port: u16, fullscreen: bool, viewer_type: &Viewer) -> Result<()> {
    let mut command = Command::new(viewer);
    command.arg("--title").arg(vm_name).arg("--spice-shared-dir").arg(publicdir);

    if fullscreen {
        command.arg("--full-screen");
    }

    match viewer_type {
        Viewer::Spicy => command.arg("--port").arg(port.to_string()),
        Viewer::Remote => command.arg("spice://localhost:".to_string() + &port.to_string()),
        _ => unreachable!(),
    };

    command.spawn().map_err(|e| anyhow!("Failed to start viewer: {}", e))?;
    Ok(())
}

pub fn create_disk_image(qemu_img: &Path, path: &Path, size: u64, format: &DiskFormat, prealloc: &PreAlloc) -> Result<()> {
    let mut qemu_img_command = Command::new(qemu_img);
    qemu_img_command
        .args(["create", "-q", "-f", format.as_ref()])
        .arg(path)
        .arg(size.to_string());

    if let Some(prealloc_args) = prealloc.qemu_img_arg(format)? {
        qemu_img_command.args(["-o", prealloc_args]);
    }

    let creation = qemu_img_command
        .output()
        .map_err(|e| anyhow!("Could not launch qemu-img to create disk image {}: {}", &path.display(), e))?;
    if !creation.status.success() {
        bail!(
            "Failed to create disk image {}: {}",
            &path.display(),
            String::from_utf8_lossy(&creation.stderr)
        );
    }
    Ok(())
}
