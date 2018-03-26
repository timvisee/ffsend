use ffsend_api::url::{ParseError, Url};

use super::clap::{App, Arg, ArgMatches, SubCommand};

use app::SEND_DEF_HOST;
use util::quit_error_msg;

/// The upload command.
pub struct CmdUpload<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CmdUpload<'a> {
    /// Build the sub command definition.
    pub fn build<'y, 'z>() -> App<'y, 'z> {
        // Build the subcommand
        #[allow(unused_mut)]
        let mut cmd = SubCommand::with_name("upload")
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
                .help("Open the share link in your browser"));

        // Optional clipboard support
        #[cfg(feature = "clipboard")] {
            cmd = cmd.arg(Arg::with_name("copy")
                .long("copy")
                .short("c")
                .help("Copy the share link to your clipboard"));
        }

        cmd
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
    ///
    /// This method parses the host into an `Url`.
    /// If the given host is invalid,
    /// the program will quit with an error message.
    pub fn host(&'a self) -> Url {
        // Get the host
        let host = self.matches.value_of("host")
            .expect("missing host");

        // Parse the URL
        match Url::parse(host) {
            Ok(url) => url,
            Err(ParseError::EmptyHost) =>
                quit_error_msg("emtpy host given"),
            Err(ParseError::InvalidPort) =>
                quit_error_msg("invalid host port"),
            Err(ParseError::InvalidIpv4Address) =>
                quit_error_msg("invalid IPv4 address in host"),
            Err(ParseError::InvalidIpv6Address) =>
                quit_error_msg("invalid IPv6 address in host"),
            Err(ParseError::InvalidDomainCharacter) =>
                quit_error_msg("host domains contains an invalid character"),
            Err(ParseError::RelativeUrlWithoutBase) =>
                quit_error_msg("host domain doesn't contain a host"),
            _ => quit_error_msg("the given host is invalid"),
        }
    }

    /// Check whether to open the file URL in the user's browser.
    pub fn open(&self) -> bool {
        self.matches.is_present("open")
    }

    /// Check whether to copy the file URL in the user's clipboard.
    #[cfg(feature = "clipboard")]
    pub fn copy(&self) -> bool {
        self.matches.is_present("copy")
    }
}
