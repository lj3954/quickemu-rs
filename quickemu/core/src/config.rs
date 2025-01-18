use crate::{
    data::*,
    error::{ConfigError, Error, MonitorError, Warning},
    qemu_args,
    utils::{EmulatorArgs, QemuArg},
};
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

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

impl Config {
    pub fn parse(file: &Path) -> Result<Self, ConfigError> {
        let contents = std::fs::read_to_string(file)?;
        let mut conf: Self = toml::from_str(&contents).map_err(ConfigError::Parse)?;
        if conf.vm_dir.is_none() {
            if conf.vm_name.is_empty() {
                let filename = file.file_name().expect("Filename should exist").to_string_lossy();
                let ext_rindex = filename.bytes().rev().position(|b| b == b'.').map_or(1, |i| i + 1);
                conf.vm_name = filename[..filename.len() - ext_rindex].to_string();
            }
            conf.vm_dir = Some(file.parent().unwrap().join(&conf.vm_name));
        } else if conf.vm_name.is_empty() {
            conf.vm_name = conf
                .vm_dir
                .as_ref()
                .unwrap()
                .file_name()
                .expect("Filename should exist")
                .to_string_lossy()
                .to_string();
        }
        Ok(conf)
    }

    pub fn send_monitor_command(&self, command: &str) -> Result<String, MonitorError> {
        self.network.send_monitor_cmd(command)
    }

    pub fn to_qemu_args(&self) -> Result<(Vec<QemuArg>, Vec<Warning>), Error> {
        #[cfg(target_arch = "x86_64")]
        self.guest.validate_cpu()?;

        qemu_args!(
            self.basic_args(),
            self.machine.cpu_args(self.guest),
            self.io.args(self.machine.arch, self.guest),
            self.io.display.audio(self.io.soundcard),
            self.io.display.args(self.guest, self.machine.arch),
        )
    }

    fn basic_args(&self) -> Result<(BasicArgs, Vec<Warning>), Error> {
        Ok((
            BasicArgs {
                slew_driftfix: matches!(self.machine.arch, Arch::X86_64 { .. }),
                pid_path: self.vm_dir.as_ref().unwrap().join(format!("{}.pid", self.vm_name)),
                vm_name: self.vm_name.clone(),
            },
            Vec::new(),
        ))
    }
}

pub(crate) struct BasicArgs {
    slew_driftfix: bool,
    pid_path: PathBuf,
    vm_name: String,
}

impl EmulatorArgs for BasicArgs {
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let mut args = Vec::with_capacity(4);

        let rtc = if self.slew_driftfix {
            "base=localtime,clock=host,driftfix=slew"
        } else {
            "base=localtime,clock=host"
        };

        args.push(Cow::Borrowed(OsStr::new("-rtc")));
        args.push(Cow::Borrowed(OsStr::new(rtc)));

        args.push(Cow::Borrowed(OsStr::new("-pidfile")));
        args.push(Cow::Owned(self.pid_path.clone().into_os_string()));

        #[cfg(target_os = "linux")]
        {
            args.push(Cow::Borrowed(OsStr::new("-name")));
            args.push(Cow::Owned(format!("{},process={}", self.vm_name, self.vm_name).into()));
        }
        args
    }
}
