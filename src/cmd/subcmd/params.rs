use clap::{App, SubCommand};

use crate::cmd::arg::{ArgDownloadLimit, ArgOwner, ArgUrl, CmdArg};

/// The params command definition.
pub struct CmdParams;

impl CmdParams {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        // Create a list of parameter arguments, of which one is required
        let param_args = [ArgDownloadLimit::name()];

        SubCommand::with_name("parameters")
            .about("Change parameters of a shared file")
            .visible_alias("params")
            .alias("param")
            .alias("parameter")
            .arg(ArgUrl::build())
            .arg(ArgOwner::build())
            .arg(ArgDownloadLimit::build().required_unless_one(&param_args))
    }
}
