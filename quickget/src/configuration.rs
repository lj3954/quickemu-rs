use anyhow::{anyhow, Result};
use quick_fetcher::{Checksum, Download, Downloader};
use quickget_ci::{Config, ConfigFile, Disk, DiskImage, Image, Source};
use std::{
    fs::File,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use which::which;

pub trait CreateConfig {
    async fn create_config(remote: Config, os: String) -> Result<(ConfigFile, String)>;
}

impl CreateConfig for ConfigFile {
    async fn create_config(remote: Config, os: String) -> Result<(ConfigFile, String)> {
        let vm_path = format!(
            "{os}{}{}-{}",
            remote.release.map(|r| "-".to_string() + &r).unwrap_or_default(),
            remote.edition.map(|e| "-".to_string() + &e).unwrap_or_default(),
            remote.arch
        );
        let vm_dir = PathBuf::from(&vm_path);
        std::fs::create_dir(&vm_dir).map_err(|e| anyhow!("Failed to create directory: {}", e))?;
        let vm_dir = vm_dir
            .canonicalize()
            .map_err(|e| anyhow!("Failed to canonicalize directory: {}", e))?;
        let mut images = Vec::new();
        let mut downloads = Vec::new();
        if let Some(iso) = remote.iso {
            let (iso_paths, iso_downloads) = convert_sources(iso, &vm_dir, vm_path.clone() + ".iso")?;
            images.extend(iso_paths.into_iter().map(Image::Iso));
            downloads.extend(iso_downloads);
        }
        if let Some(img) = remote.img {
            let (img_paths, img_downloads) = convert_sources(img, &vm_dir, vm_path.clone() + ".img")?;
            images.extend(img_paths.into_iter().map(Image::Img));
            downloads.extend(img_downloads);
        }
        if let Some(floppy) = remote.floppy {
            let (floppy_paths, floppy_downloads) = convert_sources(floppy, &vm_dir, vm_path.clone() + "-floppy.img")?;
            images.extend(floppy_paths.into_iter().map(Image::Floppy));
            downloads.extend(floppy_downloads);
        }
        if let Some(fixed_iso) = remote.fixed_iso {
            let (fixed_iso_paths, fixed_iso_downloads) = convert_sources(fixed_iso, &vm_dir, vm_path.clone() + "-cdrom.iso")?;
            images.extend(fixed_iso_paths.into_iter().map(Image::FixedIso));
            downloads.extend(fixed_iso_downloads);
        }
        let disk_images = if let Some(disks) = remote.disk_images {
            let (disk_images, disk_downloads) = handle_disks(disks, &vm_dir)?;
            downloads.extend(disk_downloads);
            disk_images
        } else {
            Vec::new()
        };

        let downloader = Downloader::new(downloads).with_progress(Default::default());
        log::debug!("Starting downloads");
        downloader.start_downloads().await?;
        Ok((
            ConfigFile {
                guest_os: remote.guest_os,
                arch: remote.arch,
                image_files: Some(images),
                disk_images,
                ..Default::default()
            },
            vm_path,
        ))
    }
}

fn convert_sources(sources: Vec<Source>, vm_dir: &Path, default_filename: String) -> Result<(Vec<PathBuf>, Vec<Download>)> {
    let mut paths = Vec::new();
    let mut downloads = Vec::new();
    for source in sources {
        match source {
            Source::FileName(file) => {
                let path = vm_dir.join(file);
                paths.push(path);
            }
            Source::Web(web) => {
                let url = reqwest::Url::parse(&web.url)?;
                let filename = web.file_name.unwrap_or_else(|| {
                    url.path_segments()
                        .and_then(|segments| segments.last())
                        .and_then(|name| if name.is_empty() { None } else { Some(name.into()) })
                        .unwrap_or(default_filename.clone())
                });
                let path = vm_dir.join(&filename);
                log::debug!("Path: {:?}", path);
                let mut dl = Download::new_from_url(url)
                    .with_output_dir(vm_dir.to_path_buf())
                    .with_filename(filename);
                if let Some(checksum) = web.checksum {
                    dl = dl.with_checksum(Checksum::new(checksum)?);
                }
                downloads.push(dl);
                paths.push(path);
            }
            Source::Custom => todo!(),
        }
    }
    Ok((paths, downloads))
}

fn convert_source(source: Source, vm_dir: &Path, default_filename: String) -> Result<(PathBuf, Option<Download>)> {
    match source {
        Source::FileName(file) => {
            let path = vm_dir.join(file);
            Ok((path, None))
        }
        Source::Web(web) => {
            let url = reqwest::Url::parse(&web.url)?;
            let filename = web.file_name.unwrap_or_else(|| {
                url.path_segments()
                    .and_then(|segments| segments.last())
                    .and_then(|name| if name.is_empty() { None } else { Some(name.into()) })
                    .unwrap_or(default_filename.clone())
            });
            let path = vm_dir.join(&filename);
            log::debug!("Path: {:?}", path);
            let mut dl = Download::new_from_url(url)
                .with_output_dir(vm_dir.to_path_buf())
                .with_filename(filename);
            if let Some(checksum) = web.checksum {
                dl = dl.with_checksum(Checksum::new(checksum)?);
            }
            Ok((path, Some(dl)))
        }
        Source::Custom => todo!(),
    }
}

fn handle_disks(disks: Vec<Disk>, vm_dir: &Path) -> Result<(Vec<DiskImage>, Vec<Download>)> {
    let mut downloads = Vec::new();
    let disk_images = disks
        .into_iter()
        .map(|disk| {
            let (path, dl) = convert_source(disk.source, vm_dir, "custom_disk.qcow2".into())?;
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
