use std::{borrow::Cow, ffi::OsString, path::Path};

use itertools::chain;
use which::which;

use crate::{
    arg,
    data::{GuestOS, MacOSRelease, Monitor, Network, NetworkType, PortForward, Serial},
    error::{Error, Warning},
    oarg,
    utils::{ArgDisplay, EmulatorArgs, LaunchFn, QemuArg},
};

mod monitor;

impl<'a> Network {
    pub(crate) fn args(&'a self, guest: GuestOS, vm_name: &'a str, publicdir: Option<&'a Path>) -> Result<(FullNetworkArgs<'a>, Option<Warning>), Error> {
        let network_args = self.inner_args(guest, vm_name, publicdir);
        Ok((
            FullNetworkArgs {
                network: network_args,
                monitor: &self.monitor,
                serial: &self.serial,
            },
            None,
        ))
    }

    fn inner_args(&'a self, guest: GuestOS, vm_name: &'a str, publicdir: Option<&'a Path>) -> NetworkArgs<'a> {
        let samba = if matches!(self.network_type, NetworkType::Nat { .. }) {
            which("smbd").ok().and(publicdir)
        } else {
            None
        };
        NetworkArgs {
            network_type: &self.network_type,
            network_device: guest.into(),
            vm_name,
            samba,
        }
    }
}

pub(crate) struct FullNetworkArgs<'a> {
    network: NetworkArgs<'a>,
    monitor: &'a Monitor,
    serial: &'a Serial,
}

impl EmulatorArgs for FullNetworkArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        chain!(self.network.display(), self.monitor.display(), self.serial.display())
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        chain!(self.network.qemu_args(), self.monitor.qemu_args(), self.serial.qemu_args())
    }
    fn launch_fns(self) -> impl IntoIterator<Item = LaunchFn> {
        chain!(self.network.launch_fns())
    }
}

struct NetworkArgs<'a> {
    network_type: &'a NetworkType,
    network_device: NetDevice,
    vm_name: &'a str,
    samba: Option<&'a Path>,
}

impl EmulatorArgs for NetworkArgs<'_> {
    fn display(&self) -> impl IntoIterator<Item = ArgDisplay> {
        let network_type = match self.network_type {
            NetworkType::None => Cow::Borrowed("Disabled"),
            NetworkType::Nat { restrict: true, .. } => Cow::Owned(format!("Restricted ({})", self.network_device)),
            NetworkType::Nat { restrict: false, .. } => Cow::Owned(format!("User ({})", self.network_device)),
            NetworkType::Bridged { bridge, .. } => Cow::Owned(format!("Bridged ({})", bridge.as_ref())),
        };

        let network_msg = ArgDisplay {
            name: Cow::Borrowed("Network"),
            value: network_type,
        };

        if let NetworkType::Nat { ssh_port, port_forwards, .. } = &self.network_type {
            let ssh_msg = match ssh_port.as_ref() {
                Some(port) => ArgDisplay {
                    name: Cow::Borrowed("SSH (Host)"),
                    value: Cow::Owned(format!("ssh {{user}}@localhost -p {port}")),
                },
                None => ArgDisplay {
                    name: Cow::Borrowed("SSH"),
                    value: Cow::Borrowed("All ports exhausted"),
                },
            };

            let samba_msg = self.samba.map(|_| ArgDisplay {
                name: Cow::Borrowed("Samba (Guest)"),
                value: Cow::Borrowed("`smb://10.0.2.4/qemu`"),
            });

            let port_forwards = port_forwards.iter().map(|PortForward { host, guest }| ArgDisplay {
                name: Cow::Borrowed("Port Forward"),
                value: Cow::Owned(format!("{host} => {guest}")),
            });
            chain!(std::iter::once(network_msg), std::iter::once(ssh_msg), samba_msg, port_forwards).collect()
        } else {
            vec![network_msg]
        }
    }
    fn qemu_args(&self) -> impl IntoIterator<Item = QemuArg> {
        match &self.network_type {
            NetworkType::None => vec![arg!("-nic"), arg!("none")],
            NetworkType::Bridged { bridge, mac_addr } => {
                let mut nic = format!("bridge,br={}", bridge.as_ref());
                if let Some(mac_addr) = mac_addr {
                    nic.push_str(&format!(",mac={}", mac_addr));
                }
                vec![arg!("-nic"), oarg!(nic)]
            }
            NetworkType::Nat { ssh_port, port_forwards, restrict } => {
                let mut net = OsString::from("user,id=nic,hostname=");
                net.push(self.vm_name);
                if let Some(ssh_port) = ssh_port.as_ref() {
                    net.push(",hostfwd=tcp::");
                    net.push(ssh_port.to_string());
                    net.push("-:22");
                }
                if *restrict {
                    net.push(",restrict=y");
                }
                if let Some(samba) = self.samba {
                    net.push(",smb=");
                    net.push(samba);
                }
                for PortForward { host, guest } in port_forwards {
                    net.push(",hostfwd=tcp::");
                    net.push(host.to_string());
                    net.push("-:");
                    net.push(guest.to_string());
                }
                if let Some(samba) = self.samba {
                    net.push(",smb=");
                    net.push(samba);
                }

                vec![arg!("-netdev"), oarg!(self.network_device.as_ref()), arg!("-net"), oarg!(net)]
            }
        }
    }
}

#[derive(derive_more::Display)]
enum NetDevice {
    E1000,
    VMXNET3,
    #[display("VirtIO Net")]
    VirtIONet,
    #[display("RTL 8139")]
    RTL8139,
}

impl AsRef<str> for NetDevice {
    fn as_ref(&self) -> &str {
        match self {
            Self::E1000 => "e1000",
            Self::VMXNET3 => "vmxnet3",
            Self::VirtIONet => "virtio-net",
            Self::RTL8139 => "rtl8139",
        }
    }
}

impl From<GuestOS> for NetDevice {
    fn from(guest: GuestOS) -> Self {
        match guest {
            GuestOS::ReactOS => Self::E1000,
            GuestOS::MacOS { release } if release >= MacOSRelease::BigSur => Self::VirtIONet,
            GuestOS::MacOS { .. } => Self::VMXNET3,
            GuestOS::Linux | GuestOS::LinuxOld | GuestOS::Solaris | GuestOS::GhostBSD | GuestOS::FreeBSD | GuestOS::GenericBSD => Self::VirtIONet,
            _ => Self::RTL8139,
        }
    }
}
