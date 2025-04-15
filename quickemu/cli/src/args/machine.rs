use clap::Parser;

#[derive(Debug, Parser)]
pub(crate) struct MachineArgs {
    #[clap(long, display_order = 1, help = "Do not commit any changes to disk/snapshot")]
    status_quo: bool,
}

impl MachineArgs {
    pub(crate) fn edit_config(self, config: &mut quickemu_core::data::Machine) {
        if self.status_quo {
            config.status_quo = true;
        }
    }
}
