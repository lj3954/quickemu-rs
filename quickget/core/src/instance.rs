use crate::{
    data_structures::{ArchiveFormat, Config, Disk, DockerSource, Source, WebSource},
    error::DLError,
    QuickgetConfig,
};
use quickemu::config::{Arch, BootType, DiskFormat, GuestOS};
use reqwest::header::HeaderMap;
use std::path::{Path, PathBuf};

pub struct QuickgetInstance {
    downloads: Vec<QGDownload>,
    docker_builds: Vec<QGDockerSource>,
    vm_path: PathBuf,
    config_data: ConfigData,
}

pub struct QGDownload {
    url: String,
    path: PathBuf,
    headers: Option<HeaderMap>,
}

pub struct QGDockerSource {
    pub url: String,
    pub privileged: bool,
    pub shared_dirs: Vec<String>,
}

struct ConfigData {
    guest_os: GuestOS,
    arch: Arch,
    iso_paths: Option<Vec<FinalSource>>,
    img_paths: Option<Vec<FinalSource>>,
    fixed_iso_paths: Option<Vec<FinalSource>>,
    floppy_paths: Option<Vec<FinalSource>>,
    disk_images: Option<Vec<FinalDisk>>,
    boot_type: BootType,
}

struct FinalDisk {
    source: FinalSource,
    size: Option<u64>,
    format: DiskFormat,
}

struct FinalSource {
    path: PathBuf,
    checksum: Option<String>,
    archive_format: Option<ArchiveFormat>,
}

struct QuickgetData<'a> {
    vm_path: &'a Path,
    os: &'a str,
    release: Option<&'a str>,
    edition: Option<&'a str>,
    arch: &'a Arch,
}

impl QuickgetInstance {
    pub fn new(config: QuickgetConfig, parent_directory: PathBuf) -> Result<Self, DLError> {
        let QuickgetConfig {
            os,
            config: Config { release, edition, arch, .. },
        } = &config;
        let vm_name = os_display('-', os, release.as_deref(), edition.as_deref(), arch);
        Self::new_with_vm_name(config, parent_directory, &vm_name)
    }
    pub fn new_with_vm_name(config: QuickgetConfig, parent_directory: PathBuf, vm_name: &str) -> Result<Self, DLError> {
        let QuickgetConfig {
            os,
            config:
                Config {
                    release,
                    edition,
                    guest_os,
                    arch,
                    iso,
                    img,
                    fixed_iso,
                    floppy,
                    disk_images,
                    boot_type,
                    ..
                },
            ..
        } = config;

        if vm_name.contains('/') {
            return Err(DLError::InvalidVMName(vm_name.to_string()));
        }

        let vm_path = parent_directory.join(vm_name);
        let data = QuickgetData {
            vm_path: &vm_path,
            os: &os,
            release: release.as_deref(),
            edition: edition.as_deref(),
            arch: &arch,
        };
        let mut dl = Vec::new();
        let mut docker = Vec::new();

        let iso_paths = iso
            .map(|iso| extract_downloads(iso, &data, ".iso", &mut dl, &mut docker))
            .transpose()?;
        let img_paths = img
            .map(|img| extract_downloads(img, &data, ".img", &mut dl, &mut docker))
            .transpose()?;
        let fixed_iso_paths = fixed_iso
            .map(|fixed_iso| extract_downloads(fixed_iso, &data, "_cdrom.iso", &mut dl, &mut docker))
            .transpose()?;
        let floppy_paths = floppy
            .map(|floppy| extract_downloads(floppy, &data, ".img", &mut dl, &mut docker))
            .transpose()?;
        let disk_images = disk_images
            .map(|disk_images| transform_disks(disk_images, &data, &mut dl, &mut docker))
            .transpose()?;

        let config_data = ConfigData {
            guest_os,
            arch,
            iso_paths,
            img_paths,
            fixed_iso_paths,
            floppy_paths,
            disk_images,
            boot_type: boot_type.unwrap_or_default(),
        };
        Ok(Self {
            downloads: dl,
            docker_builds: docker,
            vm_path,
            config_data,
        })
    }
    /// Returns all downloads. Your application must download these files, further configuration will fail otherwise.
    ///
    /// Downloads are taken out of the instance, so this function can (and must) only be called once.
    pub fn get_downloads(&mut self) -> Vec<QGDownload> {
        std::mem::take(&mut self.downloads)
    }
    /// If you want to manually handle docker builds, you can gather them with this function. It is recommended to instead use {PLACEHOLDER}.
    pub fn get_docker_builds(&mut self) -> Vec<QGDockerSource> {
        std::mem::take(&mut self.docker_builds)
    }
}

fn extract_downloads(input: Vec<Source>, data: &QuickgetData, default_file_ext: &str, dl: &mut Vec<QGDownload>, docker: &mut Vec<QGDockerSource>) -> Result<Vec<FinalSource>, DLError> {
    let vm_path = data.vm_path;

    input
        .into_iter()
        .enumerate()
        .map(|(index, source)| match source {
            Source::Web(WebSource {
                url,
                checksum,
                archive_format,
                file_name,
            }) => {
                let filename = file_name.unwrap_or_else(|| gather_filename(&url, index, default_file_ext));
                let path = vm_path.join(&filename);
                dl.push(QGDownload {
                    url,
                    path: path.clone(),
                    headers: None,
                });
                Ok(FinalSource { path, checksum, archive_format })
            }
            Source::Docker(DockerSource {
                url,
                privileged,
                shared_dirs,
                output_filename,
            }) => {
                docker.push(QGDockerSource { url, privileged, shared_dirs });
                Ok(FinalSource {
                    path: vm_path.join(&output_filename),
                    checksum: None,
                    archive_format: None,
                })
            }
            Source::FileName(filename) => Ok(FinalSource {
                path: vm_path.join(&filename),
                checksum: None,
                archive_format: None,
            }),
            Source::Custom => {
                // Windows & macOS sources will be added later on. They should generally call on external crates (e.g. rido) to gather URLs, etc.
                let QuickgetData { os, release, edition, arch, .. } = *data;
                Err(DLError::UnsupportedSource(os_display(' ', os, release, edition, arch)))
            }
        })
        .collect()
}

fn transform_disks(disk_images: Vec<Disk>, data: &QuickgetData, dl: &mut Vec<QGDownload>, docker: &mut Vec<QGDockerSource>) -> Result<Vec<FinalDisk>, DLError> {
    disk_images
        .into_iter()
        .map(|disk| {
            let file_ext = match disk.format {
                DiskFormat::Qcow2 => ".qcow2",
                DiskFormat::Qcow => ".qcow",
                DiskFormat::Raw => ".img",
                DiskFormat::Qed => ".qed",
                DiskFormat::Vdi => ".vdi",
                DiskFormat::Vpc => ".vpc",
                DiskFormat::Vhdx => ".vhdx",
            };
            let mut source = extract_downloads(vec![disk.source], data, file_ext, dl, docker)?;
            let source = source.pop().unwrap();
            Ok(FinalDisk {
                source,
                size: disk.size,
                format: disk.format,
            })
        })
        .collect::<Result<Vec<FinalDisk>, DLError>>()
}

fn gather_filename(url: &str, index: usize, extension: &str) -> String {
    url.split('/')
        .last()
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("download{index}{extension}"))
}

fn os_display(delim: char, os: &str, release: Option<&str>, edition: Option<&str>, arch: &Arch) -> String {
    let mut msg = os.to_string();
    let mut add_text = |text: &str| {
        msg.push(delim);
        msg.push_str(text);
    };
    release.map(&mut add_text);
    edition.map(&mut add_text);
    add_text(&arch.to_string());
    msg
}
