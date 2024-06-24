mod arch;
mod debian;
mod fedora_redhat;
mod independent;
mod ubuntu;

pub(crate) use arch::{ArchLinux, Archcraft, ArcoLinux, ArtixLinux, AthenaOS};
pub(crate) use debian::Antix;
pub(crate) use fedora_redhat::Alma;
pub(crate) use independent::{Alpine, NixOS};
pub(crate) use ubuntu::{Edubuntu, Elementary, Kubuntu, Lubuntu, Ubuntu, UbuntuBudgie, UbuntuCinnamon, UbuntuKylin, UbuntuMATE, UbuntuServer, UbuntuStudio, UbuntuUnity, Xubuntu};
