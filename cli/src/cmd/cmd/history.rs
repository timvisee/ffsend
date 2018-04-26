use clap::{App, SubCommand};

/// The history command definition.
pub struct CmdHistory;

impl CmdHistory {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("history")
            .about("View file history")
            .visible_alias("h")
            .alias("his")
            .alias("list")
            .alias("ls")
    }
}
