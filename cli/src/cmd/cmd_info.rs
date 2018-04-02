use ffsend_api::url::{ParseError, Url};

use super::clap::{App, Arg, ArgMatches, SubCommand};

use util::quit_error_msg;

/// The info command.
pub struct CmdInfo<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CmdInfo<'a> {
    /// Build the sub command definition.
    pub fn build<'y, 'z>() -> App<'y, 'z> {
        // Build the subcommand
        let cmd = SubCommand::with_name("info")
            .about("Fetch info about a shared file.")
            .visible_alias("i")
            .alias("information")
            .arg(Arg::with_name("URL")
                .help("The share URL")
                .required(true)
                .multiple(false))
            .arg(Arg::with_name("owner")
                .long("owner")
                .short("o")
                .alias("own")
                .alias("owner-token")
                .alias("token")
                .value_name("TOKEN")
                .help("File owner token"));

        cmd
    }

    /// Parse CLI arguments, from the given parent command matches.
    pub fn parse(parent: &'a ArgMatches<'a>) -> Option<CmdInfo<'a>> {
        parent.subcommand_matches("info")
            .map(|matches| CmdInfo { matches })
    }

    /// Get the owner token.
    pub fn owner(&'a self) -> Option<String> {
        // TODO: validate the owner token if set
        self.matches.value_of("owner")
            .map(|token| token.to_owned())
    }

    /// Get the file share URL.
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
                quit_error_msg("Emtpy host given"),
            Err(ParseError::InvalidPort) =>
                quit_error_msg("Invalid host port"),
            Err(ParseError::InvalidIpv4Address) =>
                quit_error_msg("Invalid IPv4 address in host"),
            Err(ParseError::InvalidIpv6Address) =>
                quit_error_msg("Invalid IPv6 address in host"),
            Err(ParseError::InvalidDomainCharacter) =>
                quit_error_msg("Host domains contains an invalid character"),
            Err(ParseError::RelativeUrlWithoutBase) =>
                quit_error_msg("Host domain doesn't contain a host"),
            _ => quit_error_msg("The given host is invalid"),
        }
    }
}
