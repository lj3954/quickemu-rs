use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
};

#[cfg(feature = "quickemu")]
use crate::utils::find_port;
#[cfg(feature = "quickemu")]
use serde::de::Visitor;

use super::{default_if_empty, is_default};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Network {
    #[serde(default, flatten, deserialize_with = "default_if_empty")]
    pub network_type: NetworkType,
    pub monitor: Monitor,
    #[serde(default, skip_serializing_if = "is_default")]
    pub serial: Serial,
}

#[cfg_attr(not(feature = "quickemu"), derive(Default))]
#[derive(Debug, PartialEq, Serialize, Deserialize, derive_more::AsRef)]
pub struct SSHPort(Option<u16>);
#[cfg(feature = "quickemu")]
impl Default for SSHPort {
    fn default() -> Self {
        Self(find_port(22220, 9))
    }
}

#[cfg(feature = "quickemu")]
impl Visitor<'_> for SSHPort {
    type Value = SSHPort;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a port number")
    }
    fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(Self(find_port(value, 9)))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, derive_more::AsRef)]
pub struct Bridge(String);

#[cfg(feature = "quickemu")]
impl Visitor<'_> for Bridge {
    type Value = Bridge;
    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a bridge device")
    }
    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        let networks = sysinfo::Networks::new_with_refreshed_list();
        if !networks.contains_key(value) {
            return Err(E::custom(format!("Network interface {value} could not be found.")));
        }
        Ok(Self(value.to_string()))
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct PortForward {
    pub host: u16,
    pub guest: u16,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum NetworkType {
    #[serde(alias = "none")]
    None,
    #[serde(alias = "bridged")]
    Bridged {
        bridge: Bridge,
        #[serde(default, alias = "MAC Address", alias = "macaddr", skip_serializing_if = "Option::is_none")]
        mac_addr: Option<String>,
    },
    #[serde(alias = "nat", alias = "NAT", alias = "user")]
    Nat {
        #[serde(default, skip_serializing_if = "is_default")]
        port_forwards: Vec<PortForward>,
        #[serde(default, skip_serializing_if = "is_default")]
        ssh_port: SSHPort,
        #[serde(default, skip_serializing_if = "is_default")]
        restrict: bool,
    },
}

impl Default for NetworkType {
    fn default() -> Self {
        Self::Nat {
            port_forwards: vec![],
            ssh_port: SSHPort::default(),
            restrict: false,
        }
    }
}

pub type Monitor = MonitorInner<MonitorAddr>;
pub type Serial = MonitorInner<SerialAddr>;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MonitorInner<T: MonitorArg> {
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

pub trait MonitorArg: Default + AsRef<SocketAddr> + AsMut<SocketAddr> {
    fn arg() -> &'static str;
    fn display() -> &'static str;
}

#[cfg(unix)]
impl<T: MonitorArg> Default for MonitorInner<T> {
    fn default() -> Self {
        Self::Socket { socketpath: None }
    }
}

#[cfg(not(unix))]
impl<T: MonitorArg> Default for Monitor<T> {
    fn default() -> Self {
        Self(MonitorInner::Telnet { address: T::default() })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::AsRef, derive_more::AsMut)]
pub struct MonitorAddr(SocketAddr);

impl Default for MonitorAddr {
    fn default() -> Self {
        Self(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 4440))
    }
}

impl MonitorArg for MonitorAddr {
    fn arg() -> &'static str {
        "-monitor"
    }
    fn display() -> &'static str {
        "Monitor"
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, derive_more::AsRef, derive_more::AsMut)]
pub struct SerialAddr(SocketAddr);

impl Default for SerialAddr {
    fn default() -> Self {
        Self(SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 6660))
    }
}

impl MonitorArg for SerialAddr {
    fn arg() -> &'static str {
        "-serial"
    }
    fn display() -> &'static str {
        "Serial"
    }
}
