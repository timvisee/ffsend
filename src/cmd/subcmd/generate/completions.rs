use clap::{App, Arg, Shell, SubCommand};

/// The generate completions command definition.
pub struct CmdCompletions;

impl CmdCompletions {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("completions")
            .about("Shell completions")
            .alias("completion")
            .alias("complete")
            .arg(
                Arg::with_name("SHELL")
                    .help("Shell type to generate completions for")
                    .required(true)
                    .multiple(true)
                    .takes_value(true)
                    .possible_value("all")
                    .possible_values(&Shell::variants())
                    .case_insensitive(true),
            )
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .alias("output-dir")
                    .alias("out")
                    .alias("dir")
                    .value_name("DIR")
                    .help("Shell completion files output directory"),
            )
    }
}
