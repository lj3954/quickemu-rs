mod display;
mod io;
mod machine;

use std::path::PathBuf;

use clap::Parser;
use quickemu_core::config::Config;

#[derive(Debug, Parser)]
pub(crate) struct Args {
    #[clap(long, help = "Path to the VM's config file", display_order = 0)]
    pub vm: PathBuf,
    #[clap(flatten)]
    pub io: io::IoArgs,
    #[clap(flatten)]
    pub machine: machine::MachineArgs,
}

impl Args {
    pub(crate) fn edit_config(self, config: &mut Config) {
        self.io.edit_config(&mut config.io);
        self.machine.edit_config(&mut config.machine);
    }
}
