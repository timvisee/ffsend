use chrono::Duration;
use clap::{Arg, ArgMatches};
use failure::Fail;
use ffsend_api::api::Version as ApiVersion;
use ffsend_api::config::expiry_max;

use super::{CmdArg, CmdArgFlag, CmdArgOption};
use crate::cmd::matcher::MainMatcher;
use crate::util::{
    format_duration, highlight, parse_duration, prompt_yes, quit, quit_error, ErrorHints,
};

/// The download limit argument.
pub struct ArgExpiryTime {}

impl ArgExpiryTime {
    pub fn value_checked<'a>(
        matches: &ArgMatches<'a>,
        main_matcher: &MainMatcher,
        api_version: ApiVersion,
        auth: bool,
    ) -> Option<usize> {
        // Get the expiry time value
        let mut expiry = Self::value(matches)?;

        // Get expiry time, return if allowed or when forcing
        let max = *expiry_max(api_version, auth).into_iter().max().unwrap();
        if expiry <= max || main_matcher.force() {
            return Some(expiry);
        }

        // Define function to format seconds
        let format_secs = |secs: usize| format_duration(Duration::seconds(secs as i64));

        // Prompt the user the specified expiry time is invalid
        eprintln!(
            "The expiry time must equal to or less than: {}",
            format_secs(max),
        );
        if auth {
            eprintln!("Use '{}' to force", highlight("--force"));
        } else {
            eprintln!(
                "Use '{}' to force, authenticate for higher limits",
                highlight("--force")
            );
        }

        // Ask to use maximum expiry time instead, quit if user cancelled
        if !prompt_yes(
            &format!(
                "Would you like to set expiry time to {} instead?",
                format_secs(max),
            ),
            None,
            main_matcher,
        ) {
            quit();
        }
        expiry = max;

        Some(expiry)
    }
}

impl CmdArg for ArgExpiryTime {
    fn name() -> &'static str {
        "expiry-time"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("expiry-time")
            .long("expiry-time")
            .short("e")
            .alias("expire")
            .alias("expiry")
            .value_name("TIME")
            .env("FFSEND_EXPIRY_TIME")
            .help("The file expiry time")
    }
}

impl CmdArgFlag for ArgExpiryTime {}

impl<'a> CmdArgOption<'a> for ArgExpiryTime {
    type Value = Option<usize>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        Self::value_raw(matches).map(|t| match parse_duration(t) {
            Ok(seconds) => seconds,
            Err(err) => quit_error(
                err.context("specified invalid file expiry time"),
                ErrorHints::default(),
            ),
        })
    }
}
