use crate::data::*;
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, path::PathBuf};

#[cfg(feature = "quickemu")]
use crate::{
    arg,
    error::{ConfigError, Error, MonitorError, Warning},
    full_qemu_args,
    live_vm::LiveVM,
    oarg, qemu_args,
    utils::{ArgDisplay, EmulatorArgs, LaunchFn, LaunchFnReturn, QemuArg},
};
#[cfg(feature = "quickemu")]
use std::{
    borrow::Cow,
    path::Path,
    process::{Child, Command},
    thread::JoinHandle,
};
#[cfg(feature = "quickemu")]
use which::which;

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default, skip_serializing_if = "is_default")]
    pub vm_dir: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "is_default")]
    pub vm_name: String,
    pub guest: GuestOS,
    #[serde(default, skip_serializing_if = "is_default")]
    pub machine: Machine,
    #[serde(default, skip_serializing_if = "is_default")]
    pub images: Images,
    #[serde(default, skip_serializing_if = "is_default")]
    pub network: Network,
    #[serde(default, skip_serializing_if = "is_default")]
    pub io: Io,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_args: Vec<OsString>,
}

#[cfg(feature = "quickemu")]
#[derive(Debug)]
pub struct QemuArgs {
    pub qemu_args: Vec<QemuArg>,
    pub warnings: Vec<Warning>,
    pub after_launch_fns: Vec<LaunchFn>,
    pub before_launch_fns: Vec<LaunchFn>,
    pub display: Vec<ArgDisplay>,
}

#[cfg(feature = "quickemu")]
#[derive(Debug)]
pub struct LaunchResult {
    pub display: Vec<ArgDisplay>,
    pub warnings: Vec<Warning>,
    pub threads: Vec<JoinHandle<Result<(), Error>>>,
    pub children: Vec<Child>,
}

#[cfg(feature = "quickemu")]
#[allow(clippy::large_enum_variant)]
pub enum ParsedVM {
    Config(Config),
    Live(LiveVM),
}

#[cfg(feature = "quickemu")]
impl<'a> Config {
    pub fn parse(file: &Path) -> Result<ParsedVM, ConfigError> {
        let contents = std::fs::read_to_string(file)?;
        let mut conf: Self = toml::from_str(&contents).map_err(ConfigError::Parse)?;
        if conf.vm_dir.is_none() {
            if conf.vm_name.is_empty() {
                let filename = file.file_name().expect("Filename should exist").to_string_lossy();
                let ext_rindex = filename.bytes().rposition(|b| b == b'.').unwrap_or(0);
                conf.vm_name = filename[..ext_rindex].to_string();
            }
            conf.vm_dir = Some(file.parent().unwrap().join(&conf.vm_name));
        }
        Ok(if let Some(live_vm) = LiveVM::find_active(conf.vm_dir.as_ref().unwrap())? {
            ParsedVM::Live(live_vm)
        } else {
            ParsedVM::Config(conf)
        })
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

    fn create_live_vm(&self) -> (LiveVM, PathBuf) {
        let vm_dir = self.vm_dir.as_ref().unwrap();
        let ssh_port = if let NetworkType::Nat { ssh_port, .. } = &self.network.network_type {
            *ssh_port.as_ref()
        } else {
            None
        };
        #[cfg(not(target_os = "macos"))]
        let spice_port = if let DisplayType::Spice { spice_port, .. } = self.io.display.display_type {
            Some(spice_port)
        } else {
            None
        };
        LiveVM::new(
            vm_dir,
            ssh_port,
            #[cfg(not(target_os = "macos"))]
            spice_port,
            self.network.monitor.clone(),
            self.network.serial.clone(),
        )
    }

    pub fn launch(self) -> Result<LaunchResult, Error> {
        let (live_vm, live_vm_file) = self.create_live_vm();
        let qemu_bin_str = match self.machine.arch {
            Arch::X86_64 { .. } => "qemu-system-x86_64",
            Arch::AArch64 { .. } => "qemu-system-aarch64",
            Arch::Riscv64 { .. } => "qemu-system-riscv64",
        };
        let qemu_bin = which(qemu_bin_str).map_err(|_| Error::QemuNotFound(qemu_bin_str))?;
        let mut qemu_args = self.to_full_qemu_args()?;

        let mut threads = Vec::new();
        let mut children = Vec::new();

        for launch_fn in qemu_args.before_launch_fns {
            for launch_fn_return in launch_fn.call()? {
                match launch_fn_return {
                    LaunchFnReturn::Arg(arg) => qemu_args.qemu_args.push(arg),
                    LaunchFnReturn::Display(display) => qemu_args.display.push(display),
                    LaunchFnReturn::Thread(thread) => threads.push(thread),
                    LaunchFnReturn::Process(child) => children.push(child),
                }
            }
        }

        log::debug!("Launching QEMU with args {:#?}", qemu_args.qemu_args);

        let qemu_process = Command::new(qemu_bin)
            .args(qemu_args.qemu_args)
            .spawn()
            .map_err(|e| Error::Command(qemu_bin_str, e.to_string()))?;

        live_vm.serialize(&live_vm_file, qemu_process.id())?;

        qemu_args.display.push(ArgDisplay {
            name: Cow::Borrowed("PID"),
            value: Cow::Owned(qemu_process.id().to_string()),
        });

        children.push(qemu_process);

        for launch_fn in qemu_args.after_launch_fns {
            for launch_fn_return in launch_fn.call()? {
                match launch_fn_return {
                    LaunchFnReturn::Arg(_) => panic!("Arguments should not be returned in 'after' launch fns"),
                    LaunchFnReturn::Display(display) => qemu_args.display.push(display),
                    LaunchFnReturn::Thread(thread) => threads.push(thread),
                    LaunchFnReturn::Process(child) => children.push(child),
                }
            }
        }

        Ok(LaunchResult {
            display: qemu_args.display,
            warnings: qemu_args.warnings,
            threads,
            children,
        })
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
            self.images
                .args(self.guest, vm_dir, self.machine.status_quo, self.network.monitor),
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
            self.images
                .args(self.guest, vm_dir, self.machine.status_quo, self.network.monitor),
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

#[cfg(feature = "quickemu")]
pub(crate) struct BasicArgs<'a> {
    slew_driftfix: bool,
    pid_path: PathBuf,
    vm_name: &'a str,
}

#[cfg(feature = "quickemu")]
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
