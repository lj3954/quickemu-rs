use std::{error::Error, path::Path};

use quickemu_core::config::{Config, ParsedVM};
fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().filter_level(log::LevelFilter::Warn).init();

    let config_file = std::env::args().nth(1).ok_or("No config file provided")?;
    let config = Config::parse(Path::new(&config_file)).map_err(|e| format!("Couldn't parse config: {e}"))?;

    let result = match config {
        ParsedVM::Config(config) => config.launch()?,
        ParsedVM::Live(_) => return Err("VM is already running".into()),
    };

    result.warnings.iter().for_each(|warning| log::warn!("{warning}"));
    result
        .display
        .iter()
        .for_each(|display| println!(" - {}: {}", display.name, display.value));

    for thread in result.threads {
        thread.join().expect("Couldn't join thread")?;
    }

    Ok(())
}
