mod display;
mod io;

use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct Args {
    #[clap(long, help = "Path to the VM's config file")]
    pub vm: PathBuf,
    #[clap(flatten)]
    pub io: io::IoArgs,
}
