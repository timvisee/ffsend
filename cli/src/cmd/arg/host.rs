use clap::{Arg, ArgMatches};
use ffsend_api::url::{ParseError, Url};

use app::SEND_DEF_HOST;
use super::{CmdArg, CmdArgOption};
use util::{ErrorHints, quit_error_msg};

/// The host argument.
pub struct ArgHost { }

impl CmdArg for ArgHost {
    fn name() -> &'static str {
        "host"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("host")
            .long("host")
            .short("h")
            .alias("server")
            .value_name("URL")
            .default_value(SEND_DEF_HOST)
            .help("The remote host to upload to")
    }
}

impl<'a> CmdArgOption<'a> for ArgHost {
    type Value = Url;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // Get the URL
        let url = Self::value_raw(matches).expect("missing host");

        // Parse the URL
        // TODO: improve these error messages
        match Url::parse(url) {
            Ok(url) => url,
            Err(ParseError::EmptyHost) =>
                quit_error_msg("Emtpy host given", ErrorHints::default()),
            Err(ParseError::InvalidPort) =>
                quit_error_msg("Invalid host port", ErrorHints::default()),
            Err(ParseError::InvalidIpv4Address) =>
                quit_error_msg(
                    "Invalid IPv4 address in host",
                    ErrorHints::default(),
                ),
            Err(ParseError::InvalidIpv6Address) =>
                quit_error_msg(
                    "Invalid IPv6 address in host",
                    ErrorHints::default(),
                ),
            Err(ParseError::InvalidDomainCharacter) =>
                quit_error_msg(
                    "Host domains contains an invalid character",
                    ErrorHints::default(),
                ),
            Err(ParseError::RelativeUrlWithoutBase) =>
                quit_error_msg(
                    "Host domain doesn't contain a host",
                    ErrorHints::default(),
                ),
            _ => quit_error_msg(
                    "The given host is invalid",
                    ErrorHints::default(),
                ),
        }
    }
}
