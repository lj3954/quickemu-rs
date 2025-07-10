mod args;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    env_logger::builder().filter_level(log::LevelFilter::Warn).init();
    let args = args::Args::parse();
    dbg!(&args);
    args.run()
}
