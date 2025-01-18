use std::{
    borrow::Cow,
    ffi::OsStr,
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
};

pub struct ArgDisplay {
    // e.g. "CPU"
    pub name: Cow<'static, str>,
    // e.g. "1 socket (Ryzen 5 5600), 2 cores, 4 threads"
    pub value: Cow<'static, str>,
}
pub type QemuArg = Cow<'static, OsStr>;

pub trait EmulatorArgs {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        std::iter::empty()
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg>;
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
