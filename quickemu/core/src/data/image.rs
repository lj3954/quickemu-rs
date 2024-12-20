use super::{deserialize_size, is_default};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum Image {
    #[serde(alias = "iso", alias = "ISO")]
    Iso(PathBuf),
    #[serde(alias = "fixed_iso", alias = "cdrom", alias = "CD-ROM")]
    FixedIso(PathBuf),
    #[serde(alias = "floppy")]
    Floppy(PathBuf),
    #[serde(alias = "img", alias = "IMG")]
    Img(PathBuf),
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DiskImage {
    pub path: PathBuf,
    #[serde(deserialize_with = "deserialize_size", default)]
    pub size: Option<u64>,
    #[serde(flatten, skip_serializing_if = "is_default")]
    pub format: DiskFormat,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Serialize, Deserialize)]
pub enum PreAlloc {
    #[default]
    Off,
    Metadata,
    Falloc,
    Full,
}

#[derive(Copy, Serialize, Clone, Deserialize, Debug, PartialEq)]
#[serde(tag = "format")]
pub enum DiskFormat {
    #[serde(alias = "qcow2")]
    Qcow2 { preallocation: PreAlloc },
    #[serde(alias = "raw")]
    Raw { preallocation: PreAlloc },
    #[serde(alias = "qed")]
    Qed,
    #[serde(alias = "qcow")]
    Qcow,
    #[serde(alias = "vdi")]
    Vdi,
    #[serde(alias = "vpc")]
    Vpc,
    #[serde(alias = "vhdx")]
    Vhdx,
}

impl Default for DiskFormat {
    fn default() -> Self {
        Self::Qcow2 { preallocation: PreAlloc::Off }
    }
}
impl DiskFormat {
    pub fn prealloc_arg(&self) -> &str {
        match self {
            Self::Qcow2 { preallocation } => match preallocation {
                PreAlloc::Off => "lazy_refcounts=on,preallocation=off,nocow=on",
                PreAlloc::Metadata => "lazy_refcounts=on,preallocation=metadata,nocow=on",
                PreAlloc::Falloc => "lazy_refcounts=on,preallocation=falloc,nocow=on",
                PreAlloc::Full => "lazy_refcounts=on,preallocation=full,nocow=on",
            },
            Self::Raw { preallocation } => match preallocation {
                PreAlloc::Off => "preallocation=off",
                PreAlloc::Metadata => "preallocation=metadata",
                PreAlloc::Falloc => "preallocation=falloc",
                PreAlloc::Full => "preallocation=full",
            },
            _ => "preallocation=off",
        }
    }
}
impl AsRef<str> for DiskFormat {
    fn as_ref(&self) -> &str {
        match self {
            Self::Qcow2 { .. } => "qcow2",
            Self::Raw { .. } => "raw",
            Self::Qed => "qed",
            Self::Qcow => "qcow",
            Self::Vdi => "vdi",
            Self::Vpc => "vpc",
            Self::Vhdx => "vhdx",
        }
    }
}
