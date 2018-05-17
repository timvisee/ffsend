use ffsend_api::url::Url;

use clap::ArgMatches;

use cmd::arg::{ArgOwner, ArgPassword, ArgUrl, CmdArgOption};
use super::Matcher;

/// The debug command matcher.
pub struct DebugMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a> Matcher<'a> for DebugMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches.subcommand_matches("debug")
            .map(|matches|
                 DebugMatcher {
                     matches,
                 }
            )
    }
}
