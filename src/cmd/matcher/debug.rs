use clap::ArgMatches;
use ffsend_api::url::Url;

use super::Matcher;
use crate::cmd::arg::{ArgHost, CmdArgOption};

/// The debug command matcher.
pub struct DebugMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> DebugMatcher<'a> {
    /// Get the host to upload to.
    ///
    /// This method parses the host into an `Url`.
    /// If the given host is invalid,
    /// the program will quit with an error message.
    pub fn host(&'a self) -> Url {
        ArgHost::value(self.matches)
    }
}

impl<'a> Matcher<'a> for DebugMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("debug")
            .map(|matches| DebugMatcher { matches })
    }
}
