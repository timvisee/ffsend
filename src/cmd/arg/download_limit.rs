use clap::{Arg, ArgMatches};
use ffsend_api::api::Version as ApiVersion;
use ffsend_api::config::downloads_max;

use super::{CmdArg, CmdArgFlag, CmdArgOption};
use crate::cmd::matcher::MainMatcher;
use crate::util::{highlight, prompt_yes, quit};

/// The download limit argument.
pub struct ArgDownloadLimit {}

impl ArgDownloadLimit {
    pub fn value_checked<'a>(
        matches: &ArgMatches<'a>,
        main_matcher: &MainMatcher,
        api_version: ApiVersion,
        auth: bool,
    ) -> Option<usize> {
        // Get the download value
        let mut downloads = Self::value(matches)?;

        // Get number of allowed downloads, return if allowed or when forcing
        let allowed = downloads_max(api_version, auth);
        if allowed.contains(&downloads) || main_matcher.force() {
            return Some(downloads);
        }

        // Prompt the user the specified downloads limit is invalid
        let allowed_str = allowed
            .iter()
            .map(|value| format!("{}", value))
            .collect::<Vec<_>>()
            .join(", ");
        eprintln!("The downloads limit must be one of: {}", allowed_str);
        if auth {
            eprintln!("Use '{}' to force", highlight("--force"));
        } else {
            eprintln!(
                "Use '{}' to force, authenticate for higher limits",
                highlight("--force")
            );
        }

        // Ask to use closest limit, quit if user cancelled
        let closest = closest(allowed, downloads);
        if !prompt_yes(
            &format!("Would you like to limit downloads to {} instead?", closest),
            None,
            main_matcher,
        ) {
            quit();
        }
        downloads = closest;

        Some(downloads)
    }
}

impl CmdArg for ArgDownloadLimit {
    fn name() -> &'static str {
        "download-limit"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("download-limit")
            .long("download-limit")
            .short("d")
            .alias("downloads")
            .alias("download")
            .value_name("COUNT")
            .env("FFSEND_DOWNLOAD_LIMIT")
            .help("The file download limit")
    }
}

impl CmdArgFlag for ArgDownloadLimit {}

impl<'a> CmdArgOption<'a> for ArgDownloadLimit {
    type Value = Option<usize>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // TODO: do not unwrap, report an error
        Self::value_raw(matches).map(|d| d.parse::<usize>().expect("invalid download limit"))
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
