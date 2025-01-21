use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

use super::{default_if_empty, is_default};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Network {
    #[serde(default, flatten, deserialize_with = "default_if_empty")]
    pub network_type: NetworkType,
    #[serde(default, skip_serializing_if = "is_default")]
    pub port_forwards: Vec<PortForward>,
    #[serde(default = "default_ssh_port", skip_serializing_if = "is_default_ssh")]
    pub ssh_port: u16,
    pub monitor: Monitor,
    #[serde(default, skip_serializing_if = "is_default")]
    pub serial: Serial,
}
fn default_ssh_port() -> u16 {
    22220
}
fn is_default_ssh(input: &u16) -> bool {
    *input == default_ssh_port()
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct PortForward {
    pub host: u16,
    pub guest: u16,
}

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkType {
    #[serde(alias = "none")]
    None,
    #[serde(alias = "restrict")]
    Restrict,
    #[serde(alias = "bridged")]
    Bridged {
        bridge: String,
        #[serde(default, alias = "MAC Address", alias = "macaddr", skip_serializing_if = "Option::is_none")]
        mac_addr: Option<String>,
    },
    #[default]
    #[serde(alias = "nat", alias = "NAT", alias = "user")]
    Nat,
}

pub type Monitor = MonitorInner<MonitorAddr>;
pub type Serial = MonitorInner<SerialAddr>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MonitorInner<T: Default + AsRef<SocketAddr>> {
    #[serde(alias = "none")]
    None,
    #[serde(alias = "telnet")]
    Telnet {
        #[serde(default)]
        address: T,
    },
    #[cfg(unix)]
    #[serde(alias = "socket")]
    Socket { socketpath: Option<PathBuf> },
}

#[cfg(unix)]
impl<T: AsRef<SocketAddr> + Default> Default for MonitorInner<T> {
    fn default() -> Self {
        Self::Socket { socketpath: None }
    }
}

#[cfg(not(unix))]
impl<T: AsRef<SocketAddr> + Default> Default for Monitor<T> {
    fn default() -> Self {
        Self(MonitorInner::Telnet { address: T::default() })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::AsRef)]
pub struct MonitorAddr(SocketAddr);

impl Default for MonitorAddr {
    fn default() -> Self {
        Self(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4440))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::AsRef)]
pub struct SerialAddr(SocketAddr);

impl Default for SerialAddr {
    fn default() -> Self {
        Self(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6660))
    }
}
