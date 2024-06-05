mod arch_derivatives;
mod independent;
mod ubuntu;

pub(crate) use independent::NixOS;
pub(crate) use ubuntu::{Edubuntu, Kubuntu, Lubuntu, Ubuntu, UbuntuBudgie, UbuntuCinnamon, UbuntuKylin, UbuntuMATE, UbuntuServer, UbuntuStudio, UbuntuUnity, Xubuntu};
