use std::{
    io::{Read, Write},
    net::TcpStream,
};

use crate::{
    data::{Monitor, MonitorInner},
    error::MonitorError,
};

#[cfg(unix)]
use std::os::unix::net::UnixStream;

impl Monitor {
    pub fn send_cmd(&self, command: &str) -> Result<String, MonitorError> {
        let mut response = String::new();
        match self {
            MonitorInner::Telnet { address } => {
                let mut stream = TcpStream::connect(address.as_ref()).map_err(MonitorError::Read)?;
                stream.write_all(command.as_bytes()).map_err(MonitorError::Write)?;
                stream.write_all(&[0]).map_err(MonitorError::Write)?;
                stream.shutdown(std::net::Shutdown::Write).map_err(MonitorError::Write)?;
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(1)))
                    .map_err(MonitorError::Read)?;
                stream.read_to_string(&mut response).map_err(MonitorError::Read)?;
            }
            #[cfg(unix)]
            MonitorInner::Socket { socketpath: Some(ref socketpath) } => {
                let mut stream = UnixStream::connect(socketpath).map_err(MonitorError::Read)?;
                stream.write_all(command.as_bytes()).map_err(MonitorError::Write)?;
                stream.write_all(&[0]).map_err(MonitorError::Write)?;
                stream.shutdown(std::net::Shutdown::Write).map_err(MonitorError::Write)?;
                stream
                    .set_read_timeout(Some(std::time::Duration::from_secs(1)))
                    .map_err(MonitorError::Read)?;
                stream.read_to_string(&mut response).map_err(MonitorError::Read)?;
            }
            _ => return Err(MonitorError::NoMonitor),
        }
        Ok(response)
    }
}
