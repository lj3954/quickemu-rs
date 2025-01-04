use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Could not read config file: {0}")]
    Read(#[from] std::io::Error),
    #[error("Could not parse config file: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Error, Debug, Clone)]
pub enum Error {}

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
pub enum Warning {}
