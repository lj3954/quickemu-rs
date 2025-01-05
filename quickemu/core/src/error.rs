use std::num::NonZeroUsize;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not read config file: {0}")]
    Read(#[from] std::io::Error),
    #[error("Could not parse config file: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Error, Debug, Clone)]
pub enum Error {
    #[error("CPU does not support the necessary instruction for this macOS release: {0}.")]
    Instructions(&'static str),
}

#[derive(Error, Debug)]
pub enum MonitorError {
    #[error("No monitor is enabled.")]
    NoMonitor,
    #[error("Could not write to the monitor: {0}")]
    Write(std::io::Error),
    #[error("Could not read from the monitor: {0}")]
    Read(std::io::Error),
}

#[derive(Error, Debug, Clone)]
pub enum Warning {
    #[error("macOS guests may not boot with core counts that are not powers of two. Recommended rounding: {0}.")]
    MacOSCorePow2(NonZeroUsize),
    #[error("Hardware virtualization{0} is not enabled on your CPU. Falling back to software virtualization, performance will be degraded")]
    HwVirt(&'static str),
}
