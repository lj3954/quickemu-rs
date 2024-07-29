use crate::{data_structures::OS, error::QuickgetError};
use std::{
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

const CONFIG_URL: &str = "https://github.com/quickemu-project/quickget_configs/releases/download/daily/quickget_data.json.zst";

pub struct QuickgetInstance {
    configs: Vec<OS>,
    cache_file: Option<File>,
}

impl QuickgetInstance {
    pub async fn new() -> Result<Self, QuickgetError> {
        let cache_dir = dirs::cache_dir().ok_or(QuickgetError::FailedCacheDir)?;
        Self::new_with_cache_dir(cache_dir).await
    }
    pub async fn new_with_cache_dir(cache_dir: PathBuf) -> Result<Self, QuickgetError> {
        if !cache_dir.exists() {
            return Err(QuickgetError::InvalidCacheDir(cache_dir));
        }
        let cache_file_path = cache_dir.join("quickget_data.json.zst");
        let mut cache_file = OpenOptions::new().read(true).write(true).create(true).open(&cache_file_path)?;

        let configs = if cache_file_path.is_valid()? {
            read_cache_file(&cache_file)?
        } else {
            gather_configs(Some(&mut cache_file)).await?
        };

        Ok(Self {
            configs,
            cache_file: Some(cache_file),
        })
    }
    pub async fn new_without_cache() -> Result<Self, QuickgetError> {
        gather_configs(None).await.map(|configs| Self { configs, cache_file: None })
    }
    pub async fn refresh_data(&mut self) -> Result<(), QuickgetError> {
        self.configs = gather_configs(self.cache_file.as_mut()).await?;
        Ok(())
    }
}

fn read_cache_file(file: &File) -> Result<Vec<OS>, QuickgetError> {
    let reader = zstd::stream::Decoder::new(file)?;
    serde_json::from_reader(reader).map_err(QuickgetError::from)
}

async fn gather_configs(file: Option<&mut File>) -> Result<Vec<OS>, QuickgetError> {
    let request = reqwest::get(CONFIG_URL).await?;
    let data = request.text().await?;
    if let Some(file) = file {
        file.write_all(data.as_bytes())?;
    }
    serde_json::from_str(&data).map_err(QuickgetError::from)
}

trait IsValid {
    fn is_valid(&self) -> Result<bool, QuickgetError>;
}

impl IsValid for PathBuf {
    fn is_valid(&self) -> Result<bool, QuickgetError> {
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
