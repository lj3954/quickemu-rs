use std::{borrow::Cow, ffi::OsStr};

pub struct ArgDisplay {
    // e.g. "CPU"
    pub name: Cow<'static, str>,
    // e.g. "1 socket (Ryzen 5 5600), 2 cores, 4 threads"
    pub value: Cow<'static, str>,
}
pub type QemuArg = Cow<'static, OsStr>;

pub trait EmulatorArgs {
    fn display(&self) -> Option<ArgDisplay> {
        None
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
