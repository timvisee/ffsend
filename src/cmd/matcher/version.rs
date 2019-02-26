use ffsend_api::url::Url;

use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgHost, CmdArgOption};

/// The version command matcher.
pub struct VersionMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> VersionMatcher<'a> {
    /// Get the host to probe.
    ///
    /// This method parses the host into an `Url`.
    /// If the given host is invalid,
    /// the program will quit with an error message.
    pub fn host(&'a self) -> Url {
        ArgHost::value(self.matches)
    }
}

impl<'a> Matcher<'a> for VersionMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("version")
            .map(|matches| VersionMatcher { matches })
    }
}
