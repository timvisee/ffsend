use super::clap::{App, Arg, ArgMatches, SubCommand};

/// The sub command name.
const CMD_NAME: &'static str = "upload";

/// The upload command.
pub struct CmdUpload<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CmdUpload<'a> {
    /// Build the sub command definition.
    pub fn build<'y, 'z>() -> App<'y, 'z> {
        SubCommand::with_name(CMD_NAME)
            .about("Upload files")
            .visible_alias("u")
            .visible_alias("up")
            .arg(
                Arg::with_name("FILE")
                    .help("The file to upload")
                    .required(true)
                    .multiple(false)
            )
    }

    /// Parse CLI arguments, from the given parent command matches.
    pub fn parse(parent: &'a ArgMatches<'a>) -> Option<CmdUpload<'a>> {
        parent.subcommand_matches(CMD_NAME)
            .map(|matches| CmdUpload { matches })
    }

    /// Get the selected file to upload.
    pub fn file(&'a self) -> &'a str {
        self.matches.value_of("FILE")
            .expect("please specify a file to upload")
    }
}
