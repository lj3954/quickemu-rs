use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{
    data::{Monitor, Serial},
    error::{Error, LiveVMError, MonitorError},
};

const LIVE_VM_FILENAME: &str = "quickemu-live.toml";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct LiveVM {
    pub pid: u32,
    pub ssh_port: Option<u16>,
    #[cfg(not(target_os = "macos"))]
    pub spice_port: Option<u16>,
    pub monitor: Monitor,
    pub serial: Serial,
}

impl LiveVM {
    pub(crate) fn find_active(vm_dir: &Path) -> Result<Option<Self>, LiveVMError> {
        let expected_path = vm_dir.join(LIVE_VM_FILENAME);

        if !expected_path.is_file() {
            return Ok(None);
        }

        let data = std::fs::read_to_string(&expected_path).map_err(|e| LiveVMError::LiveVMDe(e.to_string()))?;
        let live_vm: Self = toml::from_str(&data).map_err(|e| LiveVMError::LiveVMDe(e.to_string()))?;

        if live_vm.is_active() {
            Ok(Some(live_vm))
        } else {
            std::fs::remove_file(&expected_path).map_err(|e| LiveVMError::DelLiveFile(e.to_string()))?;
            Ok(None)
        }
    }
    pub fn send_monitor_cmd(&self, cmd: &str) -> Result<String, MonitorError> {
        self.monitor.send_cmd(cmd)
    }
    fn is_active(&self) -> bool {
        #[cfg(unix)]
        {
            std::process::Command::new("kill")
                .arg("-0")
                .arg(self.pid.to_string())
                .stdout(std::process::Stdio::null())
                .output()
                .is_ok_and(|output| output.status.success())
        }
    }
    pub fn kill(&self) -> Result<(), LiveVMError> {
        #[cfg(unix)]
        {
            std::process::Command::new("kill")
                .arg("-9")
                .arg(self.pid.to_string())
                .output()
                .map_err(|e| LiveVMError::VMKill(e.to_string()))?;
            Ok(())
        }
    }
    pub(crate) fn new(vm_dir: &Path, ssh_port: Option<u16>, #[cfg(not(target_os = "macos"))] spice_port: Option<u16>, monitor: Monitor, serial: Serial) -> (Self, PathBuf) {
        (
            Self {
                pid: 0,
                ssh_port,
                #[cfg(not(target_os = "macos"))]
                spice_port,
                monitor,
                serial,
            },
            vm_dir.join(LIVE_VM_FILENAME),
        )
    }
    pub(crate) fn finalize_and_serialize(&mut self, file: &Path, pid: u32) -> Result<(), Error> {
        self.pid = pid;
        if file.exists() {
            return Err(Error::FailedLiveVMSe(format!(
                "Live VM file already exists at {}",
                file.display()
            )));
        }
        let data = toml::to_string_pretty(&self).map_err(|e| Error::FailedLiveVMSe(e.to_string()))?;
        File::create(file)
            .and_then(|mut file| file.write_all(data.as_bytes()))
            .map_err(|e| Error::FailedLiveVMSe(e.to_string()))?;
        Ok(())
    }
}
