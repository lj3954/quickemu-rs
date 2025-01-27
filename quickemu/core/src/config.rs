use crate::{
    arg,
    data::*,
    error::{ConfigError, Error, MonitorError, Warning},
    full_qemu_args, oarg, qemu_args,
    utils::{ArgDisplay, EmulatorArgs, LaunchFn, LaunchFnReturn, QemuArg},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use which::which;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "is_default")]
    pub vm_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub vm_name: String,
    pub guest: GuestOS,
    #[serde(default, skip_serializing_if = "is_default")]
    pub machine: Machine,
    pub disk_images: Vec<DiskImage>,
    pub image_files: Option<Vec<Image>>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub network: Network,
    #[serde(default, skip_serializing_if = "is_default")]
    pub io: Io,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<String>,
}

#[derive(Debug)]
pub struct QemuArgs {
    pub qemu_args: Vec<QemuArg>,
    pub warnings: Vec<Warning>,
    pub launch_fns: Vec<LaunchFn>,
    pub display: Vec<ArgDisplay>,
}

impl<'a> Config {
    pub fn parse(file: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(file)?;
        let mut conf: Self = toml::from_str(&contents).map_err(ConfigError::Parse)?;
        if conf.vm_dir.is_none() {
            if conf.vm_name.is_empty() {
                let filename = file.file_name().expect("Filename should exist").to_string_lossy();
                let ext_rindex = filename.bytes().rposition(|b| b == b'.').map_or(1, |i| i + 1);
                conf.vm_name = filename[..ext_rindex].to_string();
            }
            conf.vm_dir = Some(file.parent().unwrap().join(&conf.vm_name));
        }
        Ok(conf)
    }

    fn finalize(&mut self) -> Result<(), Error> {
        if self.vm_name.is_empty() {
            self.vm_name = self
                .vm_dir
                .as_ref()
                .unwrap()
                .file_name()
                .expect("Filename should exist")
                .to_string_lossy()
                .to_string();
        }
        self.network.monitor.validate()?;
        self.network.serial.validate()?;
        #[cfg(unix)]
        {
            if let MonitorInner::Socket { socketpath } = &mut self.network.monitor {
                if socketpath.is_none() {
                    *socketpath = Some(self.vm_dir.as_ref().unwrap().join(format!("{}-monitor.socket", self.vm_name)));
                }
            }
            if let MonitorInner::Socket { socketpath } = &mut self.network.serial {
                if socketpath.is_none() {
                    *socketpath = Some(self.vm_dir.as_ref().unwrap().join(format!("{}-serial.socket", self.vm_name)));
                }
            }
        }
        Ok(())
    }

    pub fn send_monitor_command(&self, command: &str) -> Result<String, MonitorError> {
        self.network.monitor.send_cmd(command)
    }

    pub fn launch(self) -> Result<(), Error> {
        let qemu_bin = match self.machine.arch {
            Arch::X86_64 { .. } => "qemu-system-x86_64",
            Arch::AArch64 { .. } => "qemu-system-aarch64",
            Arch::Riscv64 { .. } => "qemu-system-riscv64",
        };
        let qemu_bin = which(qemu_bin).map_err(|_| Error::QemuNotFound(qemu_bin))?;

        let qemu_args = self.to_full_qemu_args()?;
        Ok(())
    }

    pub fn to_full_qemu_args(mut self) -> Result<QemuArgs, Error> {
        self.finalize()?;
        let vm_dir = self.vm_dir.as_ref().unwrap();
        #[cfg(target_arch = "x86_64")]
        self.guest.validate_cpu()?;

        let mut args = full_qemu_args!(
            self.basic_args(),
            self.machine.args(self.guest, vm_dir, &self.vm_name),
            self.io.args(self.machine.arch, self.guest, &self.vm_name),
            self.network.args(self.guest, &self.vm_name, self.io.public_dir()),
        )?;

        args.qemu_args.extend(self.extra_args.into_iter().map(|arg| oarg!(arg)));
        Ok(args)
    }

    pub fn to_qemu_args(mut self) -> Result<(Vec<QemuArg>, Vec<Warning>), Error> {
        self.finalize()?;
        let vm_dir = self.vm_dir.as_ref().unwrap();
        #[cfg(target_arch = "x86_64")]
        self.guest.validate_cpu()?;

        let (mut args, warnings) = qemu_args!(
            self.basic_args(),
            self.machine.args(self.guest, vm_dir, &self.vm_name),
            self.io.args(self.machine.arch, self.guest, &self.vm_name),
            self.network.args(self.guest, &self.vm_name, self.io.public_dir()),
        )?;
        args.extend(self.extra_args.into_iter().map(|arg| oarg!(arg)));

        Ok((args, warnings))
    }

    fn basic_args(&'a self) -> Result<(BasicArgs<'a>, Option<Warning>), Error> {
        Ok((
            BasicArgs {
                slew_driftfix: matches!(self.machine.arch, Arch::X86_64 { .. }),
                pid_path: self.vm_dir.as_ref().unwrap().join(format!("{}.pid", self.vm_name)),
                vm_name: &self.vm_name,
            },
            None,
        ))
    }
}

pub(crate) struct BasicArgs<'a> {
    slew_driftfix: bool,
    pid_path: PathBuf,
    vm_name: &'a str,
}

impl EmulatorArgs for BasicArgs<'_> {
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut args = Vec::with_capacity(4);

        let rtc = if self.slew_driftfix {
            "base=localtime,clock=host,driftfix=slew"
        } else {
            "base=localtime,clock=host"
        };

        args.push(arg!("-rtc"));
        args.push(arg!(rtc));

        args.push(arg!("-pidfile"));
        args.push(oarg!(self.pid_path.clone()));

        #[cfg(target_os = "linux")]
        {
            args.push(arg!("-name"));
            args.push(oarg!(format!("{},process={}", self.vm_name, self.vm_name)));
        }
        args
    }
}
