use clap::{App, SubCommand};

use cmd::arg::{ArgOwner, ArgUrl, CmdArg};

/// The delete command definition.
pub struct CmdDelete;

impl CmdDelete {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("delete")
            .about("Delete a shared file")
            .visible_alias("del")
            .alias("r")
            .alias("rem")
            .alias("remove")
            .arg(ArgUrl::build())
            .arg(ArgOwner::build())
    }
}
