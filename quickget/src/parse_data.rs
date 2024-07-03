use crate::data_structures::OS;
use anyhow::{anyhow, bail, Result};
use std::{
    fs::File,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use zstd::stream::Decoder;

trait IsValid {
    fn is_valid(&self) -> Result<bool>;
}

impl IsValid for PathBuf {
    fn is_valid(&self) -> Result<bool> {
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

pub async fn get_json_contents(refresh: bool) -> Result<Vec<OS>> {
    let dir = dirs::cache_dir().ok_or_else(|| anyhow!("Failed to get cache directory."))?;
    if !dir.exists() {
        bail!("Cache directory does not exist.");
    }
    let file = dir.join("quickget_data.json.zst");
    if refresh || !file.is_valid()? {
        let file = File::create(&file)?;
        let download = quick_fetcher::Download::new("https://github.com/lj3954/quickget_configs/releases/download/daily/quickget_data.json.zst")?.with_output_file(file);
        let downloader = quick_fetcher::Downloader::new(vec![download]);
        downloader.start_downloads().await?;
    }
    let file = File::open(file)?;
    let reader = Decoder::new(file)?;
    serde_json::from_reader(reader).map_err(|e| {
        anyhow!(
            "Unable to read JSON contents: {e}. Please try running {} with the `--refresh` flag to force the data to be refreshed.",
            env!("CARGO_PKG_NAME")
        )
    })
}
