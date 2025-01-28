use super::{default_if_empty, deserialize_size, is_default};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct Images {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disk_images: Vec<DiskImage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub image_files: Vec<Image>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Image {
    #[serde(alias = "ISO")]
    Iso(PathBuf),
    #[serde(alias = "Floppy")]
    Floppy(PathBuf),
    #[serde(alias = "IMG")]
    Img(PathBuf),
}

#[derive(PartialEq, Default, Debug, Serialize, Deserialize)]
pub struct DiskImage {
    pub path: PathBuf,
    #[serde(deserialize_with = "deserialize_size", default)]
    pub size: Option<u64>,
    #[serde(default, flatten, skip_serializing_if = "is_default")]
    #[serde(deserialize_with = "default_if_empty")]
    pub format: DiskFormat,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreAlloc {
    #[default]
    Off,
    Metadata,
    Falloc,
    Full,
}

#[derive(Copy, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "format")]
#[serde(rename_all = "snake_case")]
pub enum DiskFormat {
    Qcow2 {
        #[serde(default, skip_serializing_if = "is_default")]
        preallocation: PreAlloc,
    },
    Raw {
        #[serde(default, skip_serializing_if = "is_default")]
        preallocation: PreAlloc,
    },
    Qed,
    Qcow,
    Vdi,
    Vpc,
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
