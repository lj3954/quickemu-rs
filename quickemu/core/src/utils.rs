use crate::error::{Error, Warning};
use std::{borrow::Cow, ffi::OsStr};

pub type QemuArg = Cow<'static, OsStr>;

pub trait EmulatorArgs {
    fn qemu_args<A, W>(&self) -> Result<(A, W), Error>
    where
        A: IntoIterator<Item = QemuArg>,
        W: IntoIterator<Item = Warning>;
}
