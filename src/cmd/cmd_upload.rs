use super::clap::{App, Arg, ArgMatches, SubCommand};

use app::SEND_DEF_HOST;

/// The upload command.
pub struct CmdUpload<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CmdUpload<'a> {
    /// Build the sub command definition.
    pub fn build<'y, 'z>() -> App<'y, 'z> {
        SubCommand::with_name("upload")
            .about("Upload files")
            .visible_alias("u")
            .visible_alias("up")
            .arg(Arg::with_name("FILE")
                .help("The file to upload")
                .required(true)
                .multiple(false))
            .arg(Arg::with_name("host")
                .long("host")
                .short("h")
                .alias("server")
                .value_name("URL")
                .default_value(SEND_DEF_HOST)
                .help("The Send host to upload to"))
            .arg(Arg::with_name("open")
                .long("open")
                .short("o")
                .help("Open the share link in your browser"))
            .arg(Arg::with_name("c")
                .long("copy")
                .short("c")
                .help("Copy the share link to your clipboard"))
    }

    /// Parse CLI arguments, from the given parent command matches.
    pub fn parse(parent: &'a ArgMatches<'a>) -> Option<CmdUpload<'a>> {
        parent.subcommand_matches("upload")
            .map(|matches| CmdUpload { matches })
    }

    /// Get the selected file to upload.
    pub fn file(&'a self) -> &'a str {
        self.matches.value_of("FILE")
            .expect("no file specified to upload")
    }

    /// Get the host to upload to.
    pub fn host(&'a self) -> &'a str {
        self.matches.value_of("host").unwrap()
    }
}
