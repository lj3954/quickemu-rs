use std::ffi::OsString;
use anyhow::Result;
use crate::config::*;
use which::which;
use sysinfo::{System, RefreshKind, CpuRefreshKind};

impl Args {
    pub fn to_qemu_args(&self) -> Result<(OsString, Vec<String>)> {
        let qemu_bin = match &self.arch {
            Arch::x86_64 => which("qemu-system-x86_64"),
            Arch::aarch64 => which("qemu-system-aarch64"),
            Arch::riscv64 => which("qemu-system-riscv64"),
        }?;

        let qemu_version = std::process::Command::new(&qemu_bin).arg("--version").output()?;
        let friendly_ver = std::str::from_utf8(&qemu_version.stdout)?
            .split_whitespace()
            .nth(3)
            .ok_or_else(|| anyhow::anyhow!("Failed to get QEMU version."))?;

        if friendly_ver[0..1].parse::<u8>()? < 6 {
            anyhow::bail!("QEMU version 6.0.0 or higher is required. Found version {}.", friendly_ver);
        }
        
        let cpu_info = System::new_with_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::new()));

        println!("QuickemuRS {} using {} {}.", env!("CARGO_PKG_VERSION"), qemu_bin.to_str().unwrap(), friendly_ver);

        let mut qemu_args: Vec<String> = vec![];

        qemu_args.push(core_count(self.cpu_cores.0, self.cpu_cores.1, cpu_info.cpus()[0].brand()));
        qemu_args.push(ram_arg(self.ram, self.macos_release.supports_balloon()));


            

        todo!()
    }
}

fn core_count(cores: usize, threads: bool, cpu_model: &str) -> String {
    if threads && cores > 1 {
        print!(" - Using {} cores and {} threads of {cpu_model}", cores/2, cores);
        format!("-smp cores={},threads=2,sockets=1", cores/2)
    } else {
        print!(" - Using {} cores of {cpu_model}", cores);
        format!("-smp cores={},threads=1,sockets=1", cores)
    }
}

fn ram_arg(ram: u64, balloon: bool) -> String {
    println!(", {} GB of RAM.", ram as f64 / (1024.0 * 1024.0 * 1024.0));
    format!("-m {ram} {}", if balloon { "-device virtio-balloon" } else { "" })
}
