use ffsend_api::action::params::{
    PARAMS_DOWNLOAD_MIN as DOWNLOAD_MIN,
    PARAMS_DOWNLOAD_MAX as DOWNLOAD_MAX,
};
use ffsend_api::url::{ParseError, Url};

use super::clap::{App, Arg, ArgMatches, SubCommand};

use util::quit_error_msg;

/// The parameters command.
pub struct CmdParams<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CmdParams<'a> {
    /// Build the sub command definition.
    pub fn build<'y, 'z>() -> App<'y, 'z> {
        // Build a list of data parameter arguments of which one is required
        let param_args = ["download-limit"];

        // Build the subcommand
        let cmd = SubCommand::with_name("parameters")
            .about("Change parameters of a shared file.")
            .visible_alias("params")
            .alias("par")
            .alias("param")
            .alias("parameter")
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
                .help("File owner token"))
            .arg(Arg::with_name("download-limit")
                .long("download-limit")
                .short("d")
                .alias("downloads")
                .alias("download")
                .alias("down")
                .alias("dlimit")
                .alias("limit")
                .alias("lim")
                .alias("l")
                .required_unless_one(&param_args)
                .value_name("COUNT")
                .help("Set the download limit parameter"));

        cmd
    }

    /// Parse CLI arguments, from the given parent command matches.
    pub fn parse(parent: &'a ArgMatches<'a>) -> Option<CmdParams<'a>> {
        parent.subcommand_matches("parameters")
            .map(|matches| CmdParams { matches })
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

    /// Get the owner token.
    pub fn owner(&'a self) -> Option<String> {
        // TODO: validate the owner token if set
        self.matches.value_of("owner")
            .map(|token| token.to_owned())
    }

    /// Get the download limit.
    pub fn download_limit(&'a self) -> Option<u8> {
        // TODO: do not unwrap, report an error
        self.matches.value_of("download-limit")
            .map(|d| d.parse::<u8>().expect("invalid download limit"))
            .and_then(|d| {
                // Check the download limit bounds
                if d < DOWNLOAD_MIN || d > DOWNLOAD_MAX {
                    panic!(
                        "invalid download limit, must be between {} and {}",
                        DOWNLOAD_MIN,
                        DOWNLOAD_MAX,
                    );
                }

                // Return the value
                Some(d)
            })
    }
}
