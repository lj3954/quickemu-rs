use std::{
    borrow::Cow,
    ffi::OsString,
    io::{Read, Write},
    net::TcpStream,
};

use crate::{
    arg,
    data::{Monitor, MonitorArg, MonitorInner},
    error::{Error, MonitorError},
    utils::{find_port, ArgDisplay, EmulatorArgs, QemuArg},
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
                stream.write_all(b"\r\n").map_err(MonitorError::Write)?;
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
                stream.write_all(b"\r\n").map_err(MonitorError::Write)?;
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

impl<T: MonitorArg> MonitorInner<T> {
    pub(crate) fn validate(&mut self) -> Result<(), Error> {
        if let Self::Telnet { address } = self {
            let defined_port = address.as_ref().port();
            let port = find_port(defined_port, 9).ok_or(Error::UnavailablePort(defined_port))?;
            address.as_mut().set_port(port);
        }
        Ok(())
    }
}

impl<T: MonitorArg> EmulatorArgs for MonitorInner<T> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let value = match self {
            Self::None => Cow::Borrowed("None"),
            Self::Telnet { address } => Cow::Owned(format!("telnet {}", address.as_ref())),
            #[cfg(unix)]
            Self::Socket { socketpath } => Cow::Owned(format!(
                "socket {}",
                socketpath.as_ref().expect("Socketpath should be filled").display()
            )),
        };
        Some(ArgDisplay {
            name: Cow::Borrowed(T::display()),
            value,
        })
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        let arg = match self {
            Self::None => arg!("none"),
            Self::Telnet { address } => {
                let mut telnet = OsString::from("telnet:");
                telnet.push(address.as_ref().to_string());
                telnet.push(",server,nowait");
                Cow::Owned(telnet)
            }
            #[cfg(unix)]
            Self::Socket { socketpath } => {
                let mut socket = OsString::from("unix:");
                socket.push(socketpath.as_ref().expect("Socketpath should be filled"));
                socket.push(",server,nowait");
                Cow::Owned(socket)
            }
        };

        [arg!(T::arg()), arg]
    }
}
