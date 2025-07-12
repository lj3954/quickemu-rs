mod display;
mod io;
mod machine;

use anyhow::{bail, Context, Result};
use std::ffi::OsString;
use std::path::PathBuf;

use clap::Parser;
use quickemu_core::config::{Config, ParsedVM};

#[derive(Debug, Parser)]
pub(crate) struct Args {
    #[clap(long, help = "Path to the VM's config file", display_order = 0)]
    vm: PathBuf,
    #[clap(flatten)]
    io: io::IoArgs,
    #[clap(flatten)]
    machine: machine::MachineArgs,
    #[clap(long, help = "Extra arguments to pass to QEMU", display_order = 1, allow_hyphen_values = true, num_args = 1..)]
    extra_args: Vec<OsString>,
    #[clap(long, help = "Kill the running VM process", display_order = 1)]
    kill: bool,
}

enum Action {
    Launch,
    Kill,
}

impl Args {
    pub(crate) fn run(self) -> Result<()> {
        let parsed_config = self.parse_config()?;
        let action = self.get_action(&parsed_config)?;

        match action {
            Action::Launch => self.launch(parsed_config),
            Action::Kill => kill(parsed_config),
        }
    }

    fn launch(self, parsed_config: ParsedVM) -> Result<()> {
        let mut config = parsed_config.config;

        self.edit_config(&mut config);
        let result = config.launch()?;

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

    fn parse_config(&self) -> Result<ParsedVM> {
        Config::parse(&self.vm).context("Couldn't parse config")
    }

    fn get_action(&self, parsed_config: &ParsedVM) -> Result<Action> {
        match (self.kill, parsed_config.live_status.as_ref()) {
            (true, Some(_)) => Ok(Action::Kill),
            (false, None) => Ok(Action::Launch),
            (true, None) => bail!("Requested to kill VM, but it isn't running"),
            (false, Some(_)) => bail!("VM is already running"),
        }
    }

    pub(crate) fn edit_config(self, config: &mut Config) {
        self.io.edit_config(&mut config.io);
        self.machine.edit_config(&mut config.machine);
        config.extra_args.extend(self.extra_args);
        dbg!(&config);
    }
}

fn kill(parsed_config: ParsedVM) -> Result<()> {
    let live_status = parsed_config
        .live_status
        .expect("Kill action should only be returned if live status is some");

    live_status.kill()?;

    Ok(())
}
