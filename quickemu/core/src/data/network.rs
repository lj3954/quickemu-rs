use std::{net::SocketAddr, path::PathBuf};

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
    #[serde(default, skip_serializing_if = "is_default")]
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::AsRef, derive_more::AsMut)]
pub struct Monitor(MonitorInner);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::AsRef, derive_more::AsMut)]
pub struct Serial(MonitorInner);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MonitorInner {
    None,
    Telnet {
        address: SocketAddr,
    },
    #[cfg(unix)]
    Socket {
        socketpath: Option<PathBuf>,
    },
}

#[cfg(unix)]
impl Default for Monitor {
    fn default() -> Self {
        Self(MonitorInner::Socket { socketpath: None })
    }
}

#[cfg(not(unix))]
impl Default for Monitor {
    fn default() -> Self {
        Self(MonitorInner::Telnet {
            address: SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 4440),
        })
    }
}

#[cfg(unix)]
impl Default for Serial {
    fn default() -> Self {
        Self(MonitorInner::Socket { socketpath: None })
    }
}

#[cfg(not(unix))]
impl Default for Monitor {
    fn default() -> Self {
        Self(MonitorInner::Telnet {
            address: SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), 6660),
        })
    }
}
