use clap::{Arg, ArgMatches};
use ffsend_api::api::Version as ApiVersion;
use ffsend_api::config::expiry_max;

use super::{CmdArg, CmdArgFlag, CmdArgOption};
use crate::cmd::matcher::MainMatcher;
use crate::util::{highlight, prompt_yes, quit};

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
        let allowed = expiry_max(api_version, auth);
        if allowed.contains(&expiry) || main_matcher.force() {
            return Some(expiry);
        }

        // Prompt the user the specified expiry time is invalid
        let allowed_str = allowed
            .iter()
            .map(|value| format!("{}", value))
            .collect::<Vec<_>>()
            .join(", ");
        eprintln!("The expiry time must be one of: {}", allowed_str);
        if auth {
            eprintln!("Use '{}' to force", highlight("--force"));
        } else {
            eprintln!(
                "Use '{}' to force, authenticate for higher limits",
                highlight("--force")
            );
        }

        // Ask to use closest limit, quit if user cancelled
        let closest = closest(allowed, expiry);
        if !prompt_yes(
            &format!("Would you like to set expiry time to {} instead?", closest),
            None,
            main_matcher,
        ) {
            quit();
        }
        expiry = closest;

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
            .help("The file expiry time")
    }
}

impl CmdArgFlag for ArgExpiryTime {}

impl<'a> CmdArgOption<'a> for ArgExpiryTime {
    type Value = Option<usize>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // TODO: do not unwrap, report an error
        Self::value_raw(matches).map(|d| d.parse::<usize>().expect("invalid expiry time"))
    }
}

/// Find the closest value to `current` in the given `values` range.
fn closest(values: &[usize], current: usize) -> usize {
    // Own the values, sort and reverse, start with biggest first
    let mut values = values.to_vec();
    values.sort_unstable();

    // Find the closest value, return it
    *values
        .iter()
        .rev()
        .map(|value| (value, (current as i64 - *value as i64).abs()))
        .min_by_key(|value| value.1)
        .expect("failed to find closest value, none given")
        .0
}
