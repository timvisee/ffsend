pub mod completions;

use clap::{App, AppSettings, SubCommand};

use completions::CmdCompletions;

/// The generate command definition.
pub struct CmdGenerate;

impl CmdGenerate {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("generate")
            .about("Generate assets")
            .visible_alias("gen")
            .setting(AppSettings::SubcommandRequiredElseHelp)
            .subcommand(CmdCompletions::build())
    }
}
