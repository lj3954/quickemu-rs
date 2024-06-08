mod arch_derivatives;
mod fedora_redhat;
mod independent;
mod ubuntu;

pub(crate) use fedora_redhat::Alma;
pub(crate) use independent::{Alpine, NixOS};
pub(crate) use ubuntu::{Edubuntu, Kubuntu, Lubuntu, Ubuntu, UbuntuBudgie, UbuntuCinnamon, UbuntuKylin, UbuntuMATE, UbuntuServer, UbuntuStudio, UbuntuUnity, Xubuntu};
