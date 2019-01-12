use clap::{App, SubCommand};

use crate::cmd::arg::{ArgUrl, CmdArg};

/// The exists command definition.
pub struct CmdExists;

impl CmdExists {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("exists")
            .about("Check whether a remote file exists")
            .visible_alias("e")
            .alias("exist")
            .arg(ArgUrl::build())
    }
}
