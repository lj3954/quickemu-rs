use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Display, Default, Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "os")]
#[serde(rename_all = "snake_case")]
pub enum GuestOS {
    #[serde(alias = "Linux")]
    #[default]
    Linux,
    #[display("Linux (Old)")]
    #[serde(alias = "LinuxOld")]
    LinuxOld,
    #[serde(alias = "Windows")]
    Windows,
    #[serde(alias = "WindowsServer", alias = "Windows Server")]
    WindowsServer,
    #[serde(rename = "macos", alias = "macOS")]
    MacOS { release: MacOSRelease },
    #[serde(rename = "freebsd", alias = "FreeBSD")]
    FreeBSD,
    #[serde(rename = "ghostbsd", alias = "GhostBSD")]
    GhostBSD,
    #[serde(rename = "bsd", alias = "BSD")]
    GenericBSD,
    #[serde(rename = "freedos", alias = "FreeDOS")]
    FreeDOS,
    #[serde(alias = "Haiku")]
    Haiku,
    #[serde(alias = "Solaris")]
    Solaris,
    #[serde(rename = "kolibrios", alias = "KolibriOS", alias = "Kolibri OS")]
    KolibriOS,
    #[serde(rename = "reactos", alias = "ReactOS")]
    ReactOS,
    #[serde(alias = "Batocera")]
    Batocera,
}

#[derive(Display, Debug, PartialEq, Clone, Copy, PartialOrd, Serialize, Deserialize)]
pub enum MacOSRelease {
    #[serde(alias = "highsierra", alias = "High Sierra", alias = "10.13")]
    HighSierra,
    #[serde(alias = "mojave", alias = "10.14")]
    Mojave,
    #[serde(alias = "catalina", alias = "10.15")]
    Catalina,
    #[serde(alias = "bigsur", alias = "Big Sur", alias = "11")]
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
