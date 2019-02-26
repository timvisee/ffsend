use clap::{App, SubCommand};

use crate::cmd::arg::{ArgHost, CmdArg};

/// The version command definition.
pub struct CmdVersion;

impl CmdVersion {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("version")
            .about("Determine the Send server version")
            .alias("ver")
            .visible_alias("v")
            .arg(ArgHost::build())
    }
}
