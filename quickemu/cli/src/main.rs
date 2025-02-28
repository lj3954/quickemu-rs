mod args;

use std::{error::Error, path::Path};

use clap::Parser;
use quickemu_core::config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().filter_level(log::LevelFilter::Warn).init();
    let args = args::Args::parse();

    let parsed_data = Config::parse(Path::new(&args.vm)).map_err(|e| format!("Couldn't parse config: {e}"))?;

    if parsed_data.live_status.is_some() {
        return Err("VM is already running".into());
    }

    let result = parsed_data.config.launch()?;

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
