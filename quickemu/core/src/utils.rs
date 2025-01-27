use std::{
    borrow::Cow,
    ffi::OsStr,
    fmt,
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

pub trait EmulatorArgs: Sized {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        std::iter::empty()
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        std::iter::empty()
    }
    fn launch_fns(self) -> impl IntoIterator<Item = LaunchFn> {
        std::iter::empty()
    }
}

type LaunchFnReturnType = Result<Vec<LaunchFnReturn>, Error>;
pub type LaunchFnInner = Box<dyn FnOnce() -> LaunchFnReturnType>;

pub enum LaunchFn {
    Before(LaunchFnInner),
    After(LaunchFnInner),
}

impl LaunchFn {
    pub fn call(self) -> LaunchFnReturnType {
        match self {
            LaunchFn::Before(inner) => inner(),
            LaunchFn::After(inner) => inner(),
        }
    }
}

impl std::fmt::Debug for LaunchFn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaunchFn::Before(_) => write!(f, "LaunchFn::Before(_)"),
            LaunchFn::After(_) => write!(f, "LaunchFn::After(_)"),
        }
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
            let mut warnings: Vec<Warning> = Vec::new();
            let mut qemu_args: Vec<QemuArg> = Vec::new();
            $(
                {
                    let (args, warn) = $arg?;
                    warnings.extend(warn);
                    qemu_args.extend(args.qemu_args());
                }
            )*
            Ok::<_, Error>((qemu_args, warnings))
        }
    };
}

#[macro_export]
macro_rules! full_qemu_args {
    ($($arg:expr),* $(,)?) => {
        {
            let mut warnings: Vec<Warning> = Vec::new();
            let mut display: Vec<ArgDisplay> = Vec::new();
            let mut qemu_args: Vec<QemuArg> = Vec::new();
            let mut before_launch_fns = Vec::new();
            let mut after_launch_fns = Vec::new();
            $(
                {
                    let (args, warn) = $arg?;
                    warnings.extend(warn);
                    display.extend(args.display());
                    qemu_args.extend(args.qemu_args());

                    for launch_fn in args.launch_fns() {
                        match launch_fn {
                            LaunchFn::Before(_) => before_launch_fns.push(launch_fn),
                            LaunchFn::After(_) => after_launch_fns.push(launch_fn),
                        }
                    };
                }
            )*

            Ok::<_, Error>(QemuArgs {
                qemu_args,
                warnings,
                display,
                before_launch_fns,
                after_launch_fns,
            })
        }
    };
}
