use std::{
    borrow::Cow,
    ffi::OsStr,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    thread::JoinHandle,
};

#[cfg(feature = "inbuilt_commands")]
use memfd_exec::Child;
#[cfg(not(feature = "inbuilt_commands"))]
use std::process::Child;

use crate::error::Error;

#[derive(Debug)]
pub struct ArgDisplay {
    // e.g. "CPU"
    pub name: Cow<'static, str>,
    // e.g. "1 socket (Ryzen 5 5600), 2 cores, 4 threads"
    pub value: Cow<'static, str>,
}
pub type QemuArg = Cow<'static, OsStr>;
pub type LaunchFn = Box<dyn FnOnce() -> Result<Vec<LaunchFnReturn>, Error>>;

pub trait EmulatorArgs: Sized {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        std::iter::empty()
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        std::iter::empty()
    }
    fn launch_fn(self) -> Option<LaunchFn> {
        None
    }
}

#[derive(Debug)]
pub enum LaunchFnReturn {
    Thread(JoinHandle<Result<(), Error>>),
    Process(Child),
    Arg(QemuArg),
    Display(ArgDisplay),
}

pub(crate) fn plural_if(b: bool) -> &'static str {
    if b {
        "s"
    } else {
        ""
    }
}

pub(crate) fn find_port(port: u16, offset: u16) -> Option<u16> {
    (port..=port + offset).find(|port| {
        let port = SocketAddrV4::new(Ipv4Addr::LOCALHOST, *port);
        TcpListener::bind(port).is_ok()
    })
}

#[macro_export]
macro_rules! oarg {
    ($arg:expr) => {
        ::std::borrow::Cow::Owned(::std::ffi::OsString::from($arg))
    };
}

#[macro_export]
macro_rules! arg {
    ($arg:expr) => {
        ::std::borrow::Cow::Borrowed(::std::ffi::OsStr::new($arg))
    };
}

#[macro_export]
macro_rules! qemu_args {
    ($($arg:expr),* $(,)?) => {
        {
            let mut warnings = Vec::new();
            let mut qemu_args = Vec::new();
            $(
                {
                    let (args, warn) = $arg?;
                    warnings.extend(warn);
                    qemu_args.extend(args.qemu_args());
                }
            )*
            Ok((qemu_args, warnings))
        }
    };
}

#[macro_export]
macro_rules! full_qemu_args {
    ($($arg:expr),* $(,)?) => {
        {
            let mut warnings = Vec::new();
            let mut display = Vec::new();
            let mut qemu_args = Vec::new();
            let mut launch_fns = Vec::new();
            $(
                {
                    let (args, warn) = $arg?;
                    warnings.extend(warn);
                    display.extend(args.display());
                    qemu_args.extend(args.qemu_args());
                    if let Some(launch_fn) = args.launch_fn() {
                        launch_fns.push(launch_fn);
                    }
                }
            )*

            let mut launch_fn_returns = Vec::new();
            for launch_fn in launch_fns {
                launch_fn_returns.extend(launch_fn()?);
            }

            Ok(QemuArgs {
                qemu_args,
                warnings,
                display,
                launch_fn_returns,
            })
        }
    };
}
