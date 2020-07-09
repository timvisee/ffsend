use clap::{Arg, ArgMatches};
use failure::Fail;
use ffsend_api::url::Url;

use super::{CmdArg, CmdArgOption};
use crate::host::parse_host;
use crate::util::{quit_error, ErrorHints};

/// The host argument.
pub struct ArgHost {}

impl CmdArg for ArgHost {
    fn name() -> &'static str {
        "host"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("host")
            .long("host")
            .short("h")
            .value_name("URL")
            .default_value("https://send.vis.ee/")
            .env("FFSEND_HOST")
            .hide_env_values(true)
            .help("The remote host to upload to")
    }
}

impl<'a> CmdArgOption<'a> for ArgHost {
    type Value = Url;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // Get the URL
        let url = Self::value_raw(matches).expect("missing host");

        // Parse the URL
        match parse_host(&url) {
            Ok(url) => url,
            Err(err) => quit_error(
                err.context("failed to parse the given host"),
                ErrorHints::default(),
            ),
        }
    }
}
