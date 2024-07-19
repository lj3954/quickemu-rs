use crate::data_structures::{ArchiveFormat as QArchiveFormat, Config, Disk, DockerSource, Source};
use anyhow::{ensure, Context, Result};
use quick_fetcher::{ArchiveFormat, Checksum, Download, Downloader};
use quickemu::config::{ConfigFile, DiskImage, Image};
use std::{
    fs::File,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use which::which;

pub trait CreateConfig {
    async fn create_config(remote: Config, os: String, dl_threads: Option<u8>) -> Result<(ConfigFile, String)>;
}

impl CreateConfig for ConfigFile {
    async fn create_config(mut remote: Config, os: String, dl_threads: Option<u8>) -> Result<(ConfigFile, String)> {
        let vm_path = format!(
            "{os}{}{}-{}",
            remote.release.as_ref().map(|r| "-".to_string() + r).unwrap_or_default(),
            remote.edition.as_ref().map(|e| "-".to_string() + e).unwrap_or_default(),
            remote.arch,
        );
        let vm_dir = PathBuf::from(&vm_path);
        std::fs::create_dir(&vm_dir).context("Failed to create VM directory")?;
        let vm_dir = vm_dir.canonicalize().context("Failed to canonicalize directory")?;
        let mut images = Vec::new();
        let mut downloads = Vec::new();
        if let Some(iso) = remote.iso.take() {
            let (iso_paths, iso_downloads) = convert_sources(iso, &vm_dir, vm_path.clone() + ".iso", &remote)?;
            images.extend(iso_paths.into_iter().map(Image::Iso));
            downloads.extend(iso_downloads);
        }
        if let Some(img) = remote.img.take() {
            let (img_paths, img_downloads) = convert_sources(img, &vm_dir, vm_path.clone() + ".img", &remote)?;
            images.extend(img_paths.into_iter().map(Image::Img));
            downloads.extend(img_downloads);
        }
        if let Some(floppy) = remote.floppy.take() {
            let (floppy_paths, floppy_downloads) = convert_sources(floppy, &vm_dir, vm_path.clone() + "-floppy.img", &remote)?;
            images.extend(floppy_paths.into_iter().map(Image::Floppy));
            downloads.extend(floppy_downloads);
        }
        if let Some(fixed_iso) = remote.fixed_iso.take() {
            let (fixed_iso_paths, fixed_iso_downloads) = convert_sources(fixed_iso, &vm_dir, vm_path.clone() + "-cdrom.iso", &remote)?;
            images.extend(fixed_iso_paths.into_iter().map(Image::FixedIso));
            downloads.extend(fixed_iso_downloads);
        }
        let disk_images = if let Some(disks) = remote.disk_images.take() {
            let (disk_images, disk_downloads) = handle_disks(disks, &vm_dir, &remote)?;
            downloads.extend(disk_downloads);
            disk_images
        } else {
            Vec::new()
        };
        if let Some(threads) = dl_threads {
            let threads_per = threads / downloads.len() as u8;
            downloads = downloads.into_iter().map(|dl| dl.with_threads(threads_per)).collect();
        }

        let downloader = Downloader::new(downloads).with_progress(Default::default());
        log::debug!("Starting downloads");
        downloader.start_downloads().await?;
        Ok((
            ConfigFile {
                guest_os: remote.guest_os,
                arch: remote.arch,
                image_files: Some(images),
                disk_images,
                boot_type: remote.boot_type.unwrap_or_default(),
                tpm: remote.tpm.unwrap_or_default(),
                ram: remote.ram,
                ..Default::default()
            },
            vm_path,
        ))
    }
}

fn convert_sources(sources: Vec<Source>, vm_dir: &Path, default_filename: String, remote: &Config) -> Result<(Vec<PathBuf>, Vec<Download>)> {
    let mut downloads = Vec::new();
    let paths = sources
        .into_iter()
        .map(|source| {
            let (path, dl) = convert_source(source, vm_dir, default_filename.clone(), remote)?;
            if let Some(dl) = dl {
                downloads.push(dl);
            }
            Ok(path)
        })
        .collect::<Result<Vec<PathBuf>>>()?;
    Ok((paths, downloads))
}

fn convert_source(source: Source, vm_dir: &Path, default_filename: String, remote: &Config) -> Result<(PathBuf, Option<Download>)> {
    match source {
        Source::FileName(file) => {
            let path = vm_dir.join(file);
            Ok((path, None))
        }
        Source::Web(web) => {
            let url = reqwest::Url::parse(&web.url)?;
            let filename = || {
                web.file_name.unwrap_or_else(|| {
                    url.path_segments()
                        .and_then(|segments| segments.last())
                        .and_then(|name| if name.is_empty() { None } else { Some(name.into()) })
                        .unwrap_or(default_filename)
                })
            };
            let filename = match web.archive_format {
                Some(QArchiveFormat::Tar | QArchiveFormat::TarGz | QArchiveFormat::TarXz | QArchiveFormat::TarBz2 | QArchiveFormat::Zip) => None,
                Some(QArchiveFormat::Gz) => Some(filename().trim_end_matches(".gz").to_string()),
                Some(QArchiveFormat::Bz2) => Some(filename().trim_end_matches(".bz2").to_string()),
                Some(QArchiveFormat::Xz) => Some(filename().trim_end_matches(".xz").to_string()),
                _ => Some(filename()),
            };

            let mut dl = Download::new_from_url(url).with_output_dir(vm_dir.to_path_buf());
            let path = filename.as_ref().map_or(vm_dir.to_path_buf(), |f| vm_dir.join(f));
            log::debug!("Path: {:?}", path);

            if let Some(filename) = filename {
                dl = dl.with_filename(filename);
            }
            if let Some(archive_format) = web.archive_format {
                dl = dl.with_archive_format(convert_archive_format(archive_format));
            }
            if let Some(checksum) = web.checksum {
                dl = dl.with_checksum(Checksum::new(checksum)?);
            }
            Ok((path, Some(dl)))
        }
        Source::Custom => todo!(),
        Source::Docker(DockerSource {
            url,
            privileged,
            shared_dirs,
            output_filename,
        }) => {
            let mut docker = std::process::Command::new("docker");

            docker.args(["run", "--rm", "-it"]);
            docker.args(["-v", &format!("{}:/output", vm_dir.display())]);

            if let Some(ref release) = remote.release {
                docker.args(["-e", &format!("RELEASE={release}")]);
            }
            if let Some(ref edition) = remote.edition {
                docker.args(["-e", &format!("EDITION={edition}")]);
            }
            docker.args(["-e", &format!("ARCH={}", remote.arch)]);

            if privileged {
                docker.arg("--privileged");
            }
            shared_dirs.iter().for_each(|dir| {
                docker.args(["-v", &format!("{dir}:{dir}")]);
            });

            docker.arg(url);

            let status = docker.status().context("Failed to run docker")?;

            ensure!(status.success(), "Docker image build failed");
            Ok((vm_dir.join(output_filename), None))
        }
    }
}

fn handle_disks(disks: Vec<Disk>, vm_dir: &Path, remote: &Config) -> Result<(Vec<DiskImage>, Vec<Download>)> {
    let mut downloads = Vec::new();
    let disk_images = disks
        .into_iter()
        .map(|disk| {
            let (path, dl) = convert_source(disk.source, vm_dir, "custom_disk.qcow2".into(), remote)?;
            if let Some(dl) = dl {
                downloads.push(dl);
            }
            Ok(DiskImage {
                path,
                size: disk.size,
                preallocation: Default::default(),
                format: Some(disk.format),
            })
        })
        .collect::<Result<Vec<DiskImage>>>()?;
    Ok((disk_images, downloads))
}

pub fn find_quickemu() -> Option<String> {
    which("quickemu-rs")
        .ok()
        .or_else(|| {
            let path = std::env::current_exe().ok()?.with_file_name("quickemu-rs");
            if path.exists() {
                Some(path)
            } else {
                None
            }
        })
        .map(|q| format!("#!{} --vm\n\n", q.to_string_lossy()))
}

pub fn set_executable(config: &File) -> bool {
    let executable = PermissionsExt::from_mode(0o755);
    config
        .set_permissions(executable)
        .map_err(|e| log::warn!("Failed to set permissions on config file: {e}"))
        .is_ok()
}

fn convert_archive_format(input: QArchiveFormat) -> ArchiveFormat {
    match input {
        QArchiveFormat::Tar => ArchiveFormat::Tar,
        QArchiveFormat::TarBz2 => ArchiveFormat::TarBz2,
        QArchiveFormat::TarGz => ArchiveFormat::TarGz,
        QArchiveFormat::TarXz => ArchiveFormat::TarXz,
        QArchiveFormat::Xz => ArchiveFormat::Xz,
        QArchiveFormat::Gz => ArchiveFormat::Gz,
        QArchiveFormat::Bz2 => ArchiveFormat::Bz2,
        QArchiveFormat::Zip => ArchiveFormat::Zip,
    }
}
