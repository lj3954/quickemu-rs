use quickemu::config::Arch;

use crate::{
    data_structures::{Config, OS},
    error::ConfigSearchError,
};
use std::{
    fs::File,
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const CONFIG_URL: &str = "https://github.com/quickemu-project/quickget_configs/releases/download/daily/quickget_data.json.zst";

#[derive(Debug, Default)]
pub struct ConfigSearch {
    configs: Vec<OS>,
    cache_file: Option<File>,
    chosen_os: Option<OS>,
    release_is_chosen: bool,
    edition_is_chosen: bool,
}

impl ConfigSearch {
    pub async fn new() -> Result<Self, ConfigSearchError> {
        let cache_dir = dirs::cache_dir().ok_or(ConfigSearchError::FailedCacheDir)?;
        Self::new_with_cache_dir(cache_dir).await
    }
    pub async fn new_with_cache_dir(cache_dir: PathBuf) -> Result<Self, ConfigSearchError> {
        if !cache_dir.exists() {
            return Err(ConfigSearchError::InvalidCacheDir(cache_dir));
        }
        let cache_file_path = cache_dir.join("quickget_data.json.zst");

        let (configs, cache_file) = if cache_file_path.is_valid()? {
            let cache_file = File::open(&cache_file_path)?;
            (read_cache_file(&cache_file)?, cache_file)
        } else {
            let mut cache_file = File::create(&cache_file_path)?;
            (gather_configs(Some(&mut cache_file)).await?, cache_file)
        };

        Ok(Self {
            configs,
            cache_file: Some(cache_file),
            ..Default::default()
        })
    }
    pub async fn new_without_cache() -> Result<Self, ConfigSearchError> {
        gather_configs(None).await.map(|configs| Self {
            configs,
            cache_file: None,
            ..Default::default()
        })
    }
    pub async fn refresh_data(&mut self) -> Result<(), ConfigSearchError> {
        self.configs = gather_configs(self.cache_file.as_mut()).await?;
        Ok(())
    }
    pub fn get_os_list(&self) -> &[OS] {
        &self.configs
    }
    pub fn into_os_list(self) -> Vec<OS> {
        self.configs
    }
    pub fn get_chosen_os(&self) -> Option<&OS> {
        self.chosen_os.as_ref()
    }
    pub fn get_configs(&self) -> Result<&[Config], ConfigSearchError> {
        let os = self.chosen_os.as_ref().ok_or(ConfigSearchError::RequiredOS)?;
        Ok(&os.releases)
    }
    pub fn list_os_names(&self) -> Vec<&str> {
        self.configs.iter().map(|OS { name, .. }| &**name).collect()
    }
    pub fn filter_os(&mut self, os: &str) -> Result<&mut OS, ConfigSearchError> {
        let os = self
            .configs
            .drain(..)
            .find(|OS { name, .. }| name == os)
            .ok_or(ConfigSearchError::InvalidOS(os.into()))?;

        self.chosen_os = Some(os);
        Ok(self.chosen_os.as_mut().unwrap())
    }
    pub fn list_architectures(&self) -> Result<Vec<Arch>, ConfigSearchError> {
        let os = self.chosen_os.as_ref().ok_or(ConfigSearchError::RequiredOS)?;

        let architectures = [Arch::x86_64, Arch::aarch64, Arch::riscv64]
            .into_iter()
            .filter(|search_arch| os.releases.iter().any(|Config { arch, .. }| arch == search_arch))
            .collect::<Vec<Arch>>();

        Ok(architectures)
    }
    pub fn filter_arch_supported_os(&mut self, matching_arch: &Arch) -> Result<(), ConfigSearchError> {
        self.configs
            .retain(|OS { releases, .. }| releases.iter().any(|Config { arch, .. }| arch == matching_arch));

        if self.configs.is_empty() {
            return Err(ConfigSearchError::InvalidArchitecture(matching_arch.to_owned()));
        }

        Ok(())
    }
    pub fn filter_arch_configs(&mut self, matching_arch: &Arch) -> Result<(), ConfigSearchError> {
        let os = self.chosen_os.as_mut().ok_or(ConfigSearchError::RequiredOS)?;
        os.filter_arch(matching_arch)
    }
    pub fn list_releases(&self) -> Result<Vec<&str>, ConfigSearchError> {
        let os = self.chosen_os.as_ref().ok_or(ConfigSearchError::RequiredOS)?;
        let mut releases = os
            .releases
            .iter()
            .map(|Config { release, .. }| release.as_str())
            .collect::<Vec<&str>>();

        releases.sort_unstable();
        releases.dedup();

        Ok(releases)
    }
    pub fn filter_release(&mut self, matching_release: &str) -> Result<(), ConfigSearchError> {
        let os = self.chosen_os.as_mut().ok_or(ConfigSearchError::RequiredOS)?;
        self.release_is_chosen = true;
        os.filter_release(matching_release)
    }
    pub fn list_editions(&mut self) -> Result<Option<Vec<&str>>, ConfigSearchError> {
        let os = self.chosen_os.as_ref().ok_or(ConfigSearchError::RequiredOS)?;
        if !self.release_is_chosen {
            return Err(ConfigSearchError::RequiredRelease);
        }
        let mut editions = os
            .releases
            .iter()
            .filter_map(|Config { edition, .. }| edition.as_deref())
            .collect::<Vec<&str>>();

        editions.sort_unstable();
        editions.dedup();

        if editions.is_empty() {
            self.edition_is_chosen = true;
            Ok(None)
        } else {
            Ok(Some(editions))
        }
    }
    pub fn filter_edition(&mut self, matching_edition: &str) -> Result<(), ConfigSearchError> {
        let os = self.chosen_os.as_mut().ok_or(ConfigSearchError::RequiredOS)?;
        if !self.release_is_chosen {
            return Err(ConfigSearchError::RequiredRelease);
        } else if self.edition_is_chosen {
            return Err(ConfigSearchError::NoEditions);
        }
        self.edition_is_chosen = true;
        os.filter_edition(matching_edition)
    }
    pub fn pick_best_match(self) -> Result<QuickgetConfig, ConfigSearchError> {
        let mut os = self.chosen_os.ok_or(ConfigSearchError::RequiredOS)?;
        if !self.release_is_chosen {
            return Err(ConfigSearchError::RequiredRelease);
        } else if !self.edition_is_chosen {
            return Err(ConfigSearchError::RequiredEdition);
        }

        let preferred_arch = || match std::env::consts::ARCH {
            "aarch64" => Arch::aarch64,
            "riscv64" => Arch::riscv64,
            _ => Arch::x86_64,
        };

        let config = if os.releases.len() == 1 {
            os.releases.pop().unwrap()
        } else if let Some(position) = os.releases.iter().position(|Config { arch, .. }| arch == &preferred_arch()) {
            os.releases.remove(position)
        } else {
            os.releases.pop().unwrap()
        };

        Ok(QuickgetConfig { os: os.name, config })
    }
}

#[derive(Debug)]
pub struct QuickgetConfig {
    pub os: String,
    pub config: Config,
}

fn read_cache_file(file: &File) -> Result<Vec<OS>, ConfigSearchError> {
    let reader = zstd::stream::Decoder::new(file)?;
    serde_json::from_reader(reader).map_err(ConfigSearchError::from)
}

async fn gather_configs(file: Option<&mut File>) -> Result<Vec<OS>, ConfigSearchError> {
    let request = reqwest::get(CONFIG_URL).await?;
    let data = request.bytes().await?;
    if let Some(file) = file {
        file.write_all(&data)?;
    }
    let reader = zstd::stream::Decoder::new(&data[..])?;
    serde_json::from_reader(reader).map_err(ConfigSearchError::from)
}

trait IsValid {
    fn is_valid(&self) -> Result<bool, ConfigSearchError>;
}

impl IsValid for PathBuf {
    fn is_valid(&self) -> Result<bool, ConfigSearchError> {
        if self.exists() {
            if let Ok(metadata) = self.metadata() {
                if let Ok(modified) = metadata.modified() {
                    let modified_date = modified.duration_since(UNIX_EPOCH)?.as_secs() / 86400;
                    let date_today = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() / 86400;
                    return Ok(metadata.is_file() && modified_date == date_today);
                }
            }
        }
        Ok(false)
    }
}

impl OS {
    pub fn filter_release(&mut self, matching_release: &str) -> Result<(), ConfigSearchError> {
        self.releases.retain(|Config { release, .. }| release == matching_release);

        if self.releases.is_empty() {
            return Err(ConfigSearchError::InvalidRelease(
                matching_release.into(),
                self.pretty_name.clone(),
            ));
        }
        Ok(())
    }
    pub fn filter_edition(&mut self, matching_edition: &str) -> Result<(), ConfigSearchError> {
        self.releases
            .retain(|Config { edition, .. }| edition.as_ref().map_or(true, |edition| edition == matching_edition));

        if self.releases.is_empty() {
            return Err(ConfigSearchError::InvalidEdition(matching_edition.into()));
        }
        Ok(())
    }
    pub fn filter_arch(&mut self, matching_arch: &Arch) -> Result<(), ConfigSearchError> {
        self.releases.retain(|Config { arch, .. }| arch == matching_arch);

        if self.releases.is_empty() {
            return Err(ConfigSearchError::InvalidArchitecture(matching_arch.to_owned()));
        }
        Ok(())
    }
}
