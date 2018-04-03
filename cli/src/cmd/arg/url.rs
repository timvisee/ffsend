use clap::{Arg, ArgMatches};
use ffsend_api::url::{ParseError, Url};

use super::{CmdArg, CmdArgOption};
use util::quit_error_msg;

/// The URL argument.
pub struct ArgUrl { }

impl CmdArg for ArgUrl {
    fn name() -> &'static str {
        "URL"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("URL")
            .required(true)
            .multiple(false)
            .help("The share URL")
    }
}

impl<'a> CmdArgOption<'a> for ArgUrl {
    type Value = Url;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // Get the URL
        let url = Self::value_raw(matches).expect("missing URL");

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
