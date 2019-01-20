use clap::{App, SubCommand};

use crate::cmd::arg::{ArgOwner, ArgPassword, ArgUrl, CmdArg};

/// The info command definition.
pub struct CmdInfo;

impl CmdInfo {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("info")
            .about("Fetch info about a shared file")
            .visible_alias("i")
            .alias("information")
            .arg(ArgUrl::build())
            .arg(ArgOwner::build())
            .arg(ArgPassword::build())
    }
}
