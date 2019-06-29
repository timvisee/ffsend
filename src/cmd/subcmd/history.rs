use clap::{App, Arg, SubCommand};

/// The history command definition.
pub struct CmdHistory;

impl CmdHistory {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("history")
            .about("View file history")
            .visible_alias("h")
            .alias("ls")
            .arg(
                Arg::with_name("rm")
                    .long("rm")
                    .short("R")
                    .alias("remove")
                    .value_name("URL")
                    .help("Remove history entry"),
            )
            .arg(
                Arg::with_name("clear")
                    .long("clear")
                    .short("C")
                    .alias("flush")
                    .help("Clear all history"),
            )
    }
}
