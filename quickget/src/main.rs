mod configuration;
mod find_entry;
mod parse_data;

use anyhow::{anyhow, bail, Result};
use clap::Parser;
use configuration::CreateConfig;
use find_entry::FindEntry;
use parse_data::get_json_contents;
use quickget_ci::{Arch, ConfigFile};
use std::{fs::File, io::Write};

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = Args::parse();
    log::debug!("Parsed arguments: {:?}", args);
    args.validate()?;
    env_logger::Builder::new().filter_level(args.verbose.log_level_filter()).init();
    let json_data = get_json_contents(args.refresh).await?;
    log::debug!("Got JSON data: {:?}", json_data);

    let os = json_data.find_entry(&args.other, args.arch)?;
    let (config, vmpath) = ConfigFile::create_config(os, args.other.remove(0), args.download_threads).await?;
    let config_data = toml::to_string_pretty(&config).map_err(|e| anyhow!("Failed to serialize config: {}", e))?;
    let quickemu = configuration::find_quickemu();

    let config_filename = vmpath + ".toml";
    let config_file = File::create(&config_filename).map_err(|e| anyhow!("Failed to create config file: {}", e))?;

    let is_executable = configuration::set_executable(&config_file);
    let optional_msg = if quickemu.is_some() && is_executable {
        format!(" or directly execute the file with `./{}`", config_filename)
    } else {
        "".to_string()
    };

    write!(&config_file, "{}{config_data}", quickemu.unwrap_or_default())?;
    println!("To start the VM, run `quickemu-rs --vm {config_filename}`{optional_msg}");

    Ok(())
}

#[derive(Debug, Parser)]
#[clap(group = clap::ArgGroup::new("actions").multiple(false))]
struct Args {
    #[clap(short, long)]
    arch: Option<Arch>,
    #[command(flatten)]
    verbose: clap_verbosity_flag::Verbosity<clap_verbosity_flag::WarnLevel>,
    #[clap(short, long)]
    refresh: bool,
    #[clap(short, long, group = "actions")]
    open_homepage: bool,
    #[clap(short, long, group = "actions")]
    url: bool,
    #[clap(short, long, group = "actions")]
    download_only: bool,
    #[clap(short, long, group = "actions")]
    list_csv: bool,
    #[clap(long, group = "actions")]
    list_json: bool,
    #[clap(long)]
    download_threads: Option<u8>,
    other: Vec<String>,
}

impl Args {
    fn validate(&self) -> Result<()> {
        if (self.list_csv || self.list_json) && !self.other.is_empty() {
            let arg = if self.list_csv { "--list-csv" } else { "--list-json" };
            bail!("Other arguments cannot be specified alongside {arg}");
        }
        Ok(())
    }
}
