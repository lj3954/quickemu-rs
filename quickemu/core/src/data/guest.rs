use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Display, Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "os")]
pub enum GuestOS {
    #[serde(rename = "linux", alias = "Linux")]
    #[default]
    Linux,
    #[display("Linux (Old)")]
    #[serde(rename = "linux_old", alias = "LinuxOld")]
    LinuxOld,
    #[serde(rename = "windows", alias = "Windows")]
    Windows,
    #[serde(rename = "windows_server", alias = "WindowsServer")]
    WindowsServer,
    #[serde(alias = "macOS", rename = "macos")]
    MacOS { release: MacOSRelease },
    #[serde(rename = "freebsd", alias = "FreeBSD")]
    FreeBSD,
    #[serde(rename = "ghostbsd", alias = "GhostBSD")]
    GhostBSD,
    #[serde(rename = "bsd", alias = "BSD")]
    GenericBSD,
    #[serde(rename = "freedos", alias = "FreeDOS")]
    FreeDOS,
    #[serde(rename = "haiku", alias = "Haiku")]
    Haiku,
    #[serde(rename = "solaris", alias = "Solaris")]
    Solaris,
    #[serde(rename = "kolibrios", alias = "KolibriOS")]
    KolibriOS,
    #[serde(rename = "reactos", alias = "ReactOS")]
    ReactOS,
    #[serde(rename = "batocera", alias = "Batocera")]
    Batocera,
}

#[derive(Display, Debug, PartialEq, Clone, Copy, PartialOrd, Serialize, Deserialize)]
pub enum MacOSRelease {
    #[serde(alias = "highsierra", alias = "10.13")]
    HighSierra,
    #[serde(alias = "mojave", alias = "10.14")]
    Mojave,
    #[serde(alias = "catalina", alias = "10.15")]
    Catalina,
    #[serde(alias = "bigsur", alias = "11")]
    BigSur,
    #[serde(alias = "monterey", alias = "12")]
    Monterey,
    #[serde(alias = "ventura", alias = "13")]
    Ventura,
    #[serde(alias = "sonoma", alias = "14")]
    Sonoma,
    #[serde(alias = "sequoia", alias = "15")]
    Sequoia,
}
