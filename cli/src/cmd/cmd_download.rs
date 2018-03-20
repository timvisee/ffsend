use ffsend_api::url::{ParseError, Url};

use super::clap::{App, Arg, ArgMatches, SubCommand};

use app::SEND_DEF_HOST;
use util::quit_error;

/// The download command.
pub struct CmdDownload<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CmdDownload<'a> {
    /// Build the sub command definition.
    pub fn build<'y, 'z>() -> App<'y, 'z> {
        // Build the subcommand
        #[allow(unused_mut)]
        let mut cmd = SubCommand::with_name("download")
            .about("Download files")
            .visible_alias("d")
            .visible_alias("down")
            .arg(Arg::with_name("URL")
                .help("The download URL")
                .required(true)
                .multiple(false));

        cmd
    }

    /// Parse CLI arguments, from the given parent command matches.
    pub fn parse(parent: &'a ArgMatches<'a>) -> Option<CmdDownload<'a>> {
        parent.subcommand_matches("download")
            .map(|matches| CmdDownload { matches })
    }

    /// Get the URL to download the file from.
    ///
    /// This method parses the URL into an `Url`.
    /// If the given URL is invalid,
    /// the program will quit with an error message.
    pub fn url(&'a self) -> Url {
        // Get the host
        let url = self.matches.value_of("URL")
            .expect("missing URL");

        // Parse the URL
        // TODO: improve these error messages
        match Url::parse(url) {
            Ok(url) => url,
            Err(ParseError::EmptyHost) =>
                quit_error("emtpy host given"),
            Err(ParseError::InvalidPort) =>
                quit_error("invalid host port"),
            Err(ParseError::InvalidIpv4Address) =>
                quit_error("invalid IPv4 address in host"),
            Err(ParseError::InvalidIpv6Address) =>
                quit_error("invalid IPv6 address in host"),
            Err(ParseError::InvalidDomainCharacter) =>
                quit_error("host domains contains an invalid character"),
            Err(ParseError::RelativeUrlWithoutBase) =>
                quit_error("host domain doesn't contain a host"),
            _ => quit_error("the given host is invalid"),
        }
    }
}
