use anyhow::{anyhow, bail, Result};
use dirs::cache_dir;
use quickget_ci::OS;
use std::{
    fs::read_to_string,
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

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

pub fn get_json_contents() -> Result<String> {
    let dir = cache_dir().unwrap_or(PathBuf::from("~/.cache"));
    let file = dir.join("quickget_entries.json");
    if file.is_valid()? {
        read_to_string(&file).map_err(|e| anyhow!("Failed to read contents of cache file: {}", e))
    } else {
        todo!()
    }
}
