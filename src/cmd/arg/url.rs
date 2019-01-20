use clap::{Arg, ArgMatches};
use failure::Fail;
use ffsend_api::url::Url;

use super::{CmdArg, CmdArgOption};
use crate::host::parse_host;
use crate::util::{quit_error, ErrorHints};

/// The URL argument.
pub struct ArgUrl {}

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
        match parse_host(&url) {
            Ok(url) => url,
            Err(err) => quit_error(
                err.context("failed to parse the given share URL"),
                ErrorHints::default(),
            ),
        }
    }
}
