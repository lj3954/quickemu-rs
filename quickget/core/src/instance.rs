use crate::{
    data_structures::{ArchiveFormat, Config, Disk, DockerSource, Source, WebSource},
    error::DLError,
    QuickgetConfig,
};
use quickemu::{
    config::{Arch, BootType, ConfigFile, DiskFormat, DiskImage, GuestOS, Image},
    config_parse::BYTES_PER_GB,
};
use reqwest::header::HeaderMap;
use sha2::Digest;
use std::{
    fs::File,
    io::Write,
    num::NonZeroUsize,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Debug, Clone)]
pub struct QuickgetInstance {
    downloads: Vec<QGDownload>,
    docker_builds: Vec<QGDockerSource>,
    vm_path: PathBuf,
    config_file_path: PathBuf,
    config_data: ConfigData,
    pub release: String,
    pub edition: Option<String>,
}

#[derive(Debug, Clone)]
pub struct QGDownload {
    pub url: String,
    pub path: PathBuf,
    pub headers: Option<HeaderMap>,
}

#[derive(Debug, Clone)]
pub struct QGDockerSource {
    pub url: String,
    pub privileged: bool,
    pub shared_dirs: Vec<String>,
}

#[derive(Debug, Clone)]
struct ConfigData {
    guest_os: GuestOS,
    arch: Arch,
    iso_paths: Option<Vec<FinalSource>>,
    img_paths: Option<Vec<FinalSource>>,
    fixed_iso_paths: Option<Vec<FinalSource>>,
    floppy_paths: Option<Vec<FinalSource>>,
    disk_images: Option<Vec<FinalDisk>>,
    boot_type: BootType,
    tpm: bool,
    cpu_cores: Option<NonZeroUsize>,
    ram: Option<u64>,
}

#[derive(Debug, Clone)]
struct FinalDisk {
    source: FinalSource,
    size: Option<u64>,
    format: DiskFormat,
}

#[derive(Debug, Clone)]
struct FinalSource {
    path: PathBuf,
    checksum: Option<String>,
    archive_format: Option<ArchiveFormat>,
}

struct QuickgetData<'a> {
    vm_path: &'a Path,
    os: &'a str,
    release: &'a str,
    edition: Option<&'a str>,
    arch: &'a Arch,
}

impl QuickgetInstance {
    pub fn new(config: QuickgetConfig, parent_directory: PathBuf) -> Result<Self, DLError> {
        let QuickgetConfig {
            os,
            config: Config { release, edition, arch, .. },
        } = &config;
        let vm_name = os_display('-', os, release, edition.as_deref(), arch);
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
                    tpm,
                    ..
                },
            ..
        } = config;

        if vm_name.contains('/') {
            return Err(DLError::InvalidVMName(vm_name.to_string()));
        }

        let vm_path = parent_directory.join(vm_name);
        let config_file_path = parent_directory.join(format!("{}.toml", vm_name));
        let data = QuickgetData {
            vm_path: &vm_path,
            os: &os,
            release: release.as_str(),
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
            tpm: tpm.unwrap_or_default(),
            cpu_cores: None,
            ram: None,
        };
        Ok(Self {
            downloads: dl,
            docker_builds: docker,
            vm_path,
            config_file_path,
            config_data,
            release,
            edition,
        })
    }
    /// Returns all downloads. Your application must download these files, further configuration will fail otherwise.
    ///
    /// Downloads are taken out of the instance, so this function can (and must) only be called once.
    pub fn get_downloads(&mut self) -> Vec<QGDownload> {
        std::mem::take(&mut self.downloads)
    }
    /// If you want to manually handle docker builds, you can gather them with this function. It is recommended to instead use get_docker_commands.
    pub fn get_docker_builds(&mut self) -> Vec<QGDockerSource> {
        std::mem::take(&mut self.docker_builds)
    }
    pub fn get_docker_commands(&mut self) -> Vec<Command> {
        self.docker_builds
            .drain(..)
            .map(|docker_build| {
                let mut command = std::process::Command::new("docker");

                command.args(["run", "--rm", "-it"]);
                command.args(["-v", &format!("{}:/output", self.vm_path.display())]);

                command.args(["-e", &format!("RELEASE={}", self.release)]);
                if let Some(ref edition) = self.edition {
                    command.args(["-e", &format!("EDITION={edition}")]);
                }
                command.args(["-e", &format!("ARCH={}", self.config_data.arch)]);

                if docker_build.privileged {
                    command.arg("--privileged");
                }
                docker_build.shared_dirs.iter().for_each(|dir| {
                    command.args(["-v", &format!("{dir}:{dir}")]);
                });
                command.arg(docker_build.url);

                command
            })
            .collect()
    }
    pub fn get_recommended_cpu_cores() -> usize {
        match num_cpus::get() {
            32.. => 16,
            16.. => 8,
            8.. => 4,
            4.. => 2,
            _ => 1,
        }
    }
    pub fn create_vm_dir(&self, overwrite: bool) -> Result<(), DLError> {
        if self.vm_path.exists() {
            if overwrite {
                std::fs::remove_dir_all(&self.vm_path)?;
            } else {
                return Err(DLError::DirAlreadyExists(self.vm_path.to_owned()));
            }
        }
        std::fs::create_dir_all(&self.vm_path)?;
        Ok(())
    }
    pub fn get_total_cpu_cores() -> usize {
        num_cpus::get()
    }
    pub fn set_cpu_cores(&mut self, cores: NonZeroUsize) {
        self.config_data.cpu_cores = Some(cores);
    }
    pub fn get_cpu_cores(&self) -> Option<usize> {
        self.config_data.cpu_cores.map(NonZeroUsize::get)
    }
    pub fn get_recommended_ram() -> u64 {
        let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_memory(sysinfo::MemoryRefreshKind::new().with_ram()));
        let ram = system.total_memory();
        match ram / (1000 * 1000 * 1000) {
            128.. => 32 * BYTES_PER_GB,
            64.. => 16 * BYTES_PER_GB,
            16.. => 8 * BYTES_PER_GB,
            8.. => 4 * BYTES_PER_GB,
            _ => ram,
        }
    }
    pub fn get_total_ram() -> u64 {
        let system = sysinfo::System::new_with_specifics(sysinfo::RefreshKind::new().with_memory(sysinfo::MemoryRefreshKind::new().with_ram()));
        system.total_memory()
    }
    pub fn set_ram(&mut self, ram: u64) {
        self.config_data.ram = Some(ram);
    }
    pub fn get_ram(&self) -> Option<u64> {
        self.config_data.ram
    }
    pub fn create_config(self) -> Result<File, DLError> {
        let image_files = {
            let mut images = Vec::new();
            if let Some(iso_sources) = self.config_data.iso_paths {
                for iso in iso_sources {
                    images.push(Image::Iso(finalize_source(iso, true)?));
                }
            }
            if let Some(img_sources) = self.config_data.img_paths {
                for img in img_sources {
                    images.push(Image::Img(finalize_source(img, true)?));
                }
            }
            if let Some(fixed_iso_sources) = self.config_data.fixed_iso_paths {
                for fixed_iso in fixed_iso_sources {
                    images.push(Image::FixedIso(finalize_source(fixed_iso, true)?));
                }
            }
            if let Some(floppy_sources) = self.config_data.floppy_paths {
                for floppy in floppy_sources {
                    images.push(Image::Floppy(finalize_source(floppy, true)?));
                }
            }
            (!images.is_empty()).then_some(images)
        };

        let disk_images = self
            .config_data
            .disk_images
            .into_iter()
            .flatten()
            .map(|disk| {
                let path = finalize_source(disk.source, false)?;
                Ok(DiskImage {
                    path,
                    size: disk.size,
                    format: Some(disk.format),
                    preallocation: quickemu::config::PreAlloc::Off,
                })
            })
            .collect::<Result<Vec<_>, DLError>>()?;

        let config = ConfigFile {
            guest_os: self.config_data.guest_os,
            arch: self.config_data.arch,
            boot_type: self.config_data.boot_type,
            cpu_cores: self.config_data.cpu_cores,
            ram: self.config_data.ram,
            tpm: self.config_data.tpm,
            disk_images,
            image_files,
            ..Default::default()
        };
        let mut config_file = File::create(&self.config_file_path)?;

        let shebang = which::which("quickemu-rs")
            .ok()
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .map(|path| path.with_file_name("quickemu-rs"))
                    .filter(|path| path.exists())
            })
            .map(|path| format!("#!{} --vm\n", path.to_string_lossy()));

        if shebang.is_some() {
            let _ = config_file.set_permissions(PermissionsExt::from_mode(0o755));
        }

        let serialized_config = toml::to_string_pretty(&config)?;
        writeln!(config_file, "{}{serialized_config}", shebang.unwrap_or_default())?;

        Ok(config_file)
    }
}

fn finalize_source(source: FinalSource, check_exists: bool) -> Result<PathBuf, DLError> {
    let FinalSource { mut path, checksum, archive_format } = source;
    if !path.exists() && check_exists {
        return Err(DLError::DownloadError(path));
    }
    let bytes = if checksum.is_some() || archive_format.is_some() { Some(std::fs::read(&path)?) } else { None };
    if let Some(checksum) = checksum {
        let bytes = bytes.as_ref().unwrap();
        let computed_hash = match checksum.len() {
            32 => format!("{:x}", md5::Md5::digest(bytes)),
            40 => format!("{:x}", sha1::Sha1::digest(bytes)),
            64 => format!("{:x}", sha2::Sha256::digest(bytes)),
            128 => format!("{:x}", sha2::Sha512::digest(bytes)),
            _ => unreachable!(),
        };
        if computed_hash != checksum {
            return Err(DLError::FailedValidation(checksum, computed_hash));
        }
    }
    if let Some(archive_format) = archive_format {
        let bytes = bytes.unwrap();
        let bytes = bytes.as_slice();
        let archive_ext = match archive_format {
            ArchiveFormat::Bz2 => Some("bz2"),
            ArchiveFormat::Gz => Some("gz"),
            ArchiveFormat::Xz => Some("xz"),
            _ => None,
        };
        if let Some(ext) = archive_ext {
            let file_name = path.file_name().unwrap().to_str().unwrap();
            if path.ends_with(ext) {
                let file_name = file_name[..file_name.len() - ext.len() - 1].to_string();
                path.set_file_name(file_name);
            }
        }
        let mut file = std::fs::File::create_new(&path)?;
        match archive_format {
            ArchiveFormat::Bz2 => {
                let mut decompressor = bzip2::read::BzDecoder::new(bytes);
                std::io::copy(&mut decompressor, &mut file)?;
            }
            ArchiveFormat::Gz => {
                let mut decompressor = flate2::read::GzDecoder::new(bytes);
                std::io::copy(&mut decompressor, &mut file)?;
            }
            ArchiveFormat::Xz => {
                let mut decompressor = liblzma::read::XzDecoder::new(bytes);
                std::io::copy(&mut decompressor, &mut file)?;
            }
            _ => unimplemented!("Unsupported archive format"),
        }
    }
    Ok(path)
}

fn convert_download(source: Source, data: &QuickgetData, default_file_ext: &str, index: usize, dl: &mut Vec<QGDownload>, docker: &mut Vec<QGDockerSource>) -> Result<FinalSource, DLError> {
    let vm_path = data.vm_path;
    match source {
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
            if let Some(ref checksum) = checksum {
                if ![32, 40, 64, 128].contains(&checksum.len()) {
                    return Err(DLError::InvalidChecksum(checksum.to_owned()));
                }
            }
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
    }
}

fn extract_downloads(input: Vec<Source>, data: &QuickgetData, default_file_ext: &str, dl: &mut Vec<QGDownload>, docker: &mut Vec<QGDockerSource>) -> Result<Vec<FinalSource>, DLError> {
    input
        .into_iter()
        .enumerate()
        .map(|(index, source)| convert_download(source, data, default_file_ext, index, dl, docker))
        .collect()
}

fn transform_disks(disk_images: Vec<Disk>, data: &QuickgetData, dl: &mut Vec<QGDownload>, docker: &mut Vec<QGDockerSource>) -> Result<Vec<FinalDisk>, DLError> {
    disk_images
        .into_iter()
        .enumerate()
        .map(|(index, disk)| {
            let file_ext = match disk.format {
                DiskFormat::Qcow2 => ".qcow2",
                DiskFormat::Qcow => ".qcow",
                DiskFormat::Raw => ".img",
                DiskFormat::Qed => ".qed",
                DiskFormat::Vdi => ".vdi",
                DiskFormat::Vpc => ".vpc",
                DiskFormat::Vhdx => ".vhdx",
            };
            let source = convert_download(disk.source, data, file_ext, index, dl, docker)?;
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

fn os_display(delim: char, os: &str, release: &str, edition: Option<&str>, arch: &Arch) -> String {
    let mut msg = format!("{os}{delim}{release}");
    if let Some(edition) = edition {
        msg.push(delim);
        msg.push_str(edition);
    }
    msg.push(delim);
    msg.push_str(arch.as_ref());
    msg
}
