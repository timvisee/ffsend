use clap::{App, SubCommand};

/// The debug command definition.
pub struct CmdDebug;

impl CmdDebug {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("debug")
            .about("View debug information")
            .visible_alias("dbg")
    }
}
