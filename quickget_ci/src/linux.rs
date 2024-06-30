mod arch;
mod debian;
mod fedora_redhat;
mod independent;
mod ubuntu;

pub(crate) use arch::{manjaro::BigLinux, ArchLinux, Archcraft, ArcoLinux, ArtixLinux, AthenaOS, BlendOS};
pub(crate) use debian::Antix;
pub(crate) use fedora_redhat::{Alma, Bazzite};
pub(crate) use independent::{Alpine, Batocera, NixOS};
pub(crate) use ubuntu::{Bodhi, Edubuntu, Elementary, Kubuntu, Lubuntu, Ubuntu, UbuntuBudgie, UbuntuCinnamon, UbuntuKylin, UbuntuMATE, UbuntuServer, UbuntuStudio, UbuntuUnity, Xubuntu};
