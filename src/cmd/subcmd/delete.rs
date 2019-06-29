use clap::{App, SubCommand};

use crate::cmd::arg::{ArgOwner, ArgUrl, CmdArg};

/// The delete command definition.
pub struct CmdDelete;

impl CmdDelete {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("delete")
            .about("Delete a shared file")
            .visible_alias("del")
            .visible_alias("rm")
            .arg(ArgUrl::build())
            .arg(ArgOwner::build())
    }
}
