use ffsend_api::url::Url;

use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgUrl, CmdArgOption};

/// The exists command matcher.
pub struct ExistsMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> ExistsMatcher<'a> {
    /// Get the file share URL.
    ///
    /// This method parses the URL into an `Url`.
    /// If the given URL is invalid,
    /// the program will quit with an error message.
    pub fn url(&'a self) -> Url {
        ArgUrl::value(self.matches)
    }
}

impl<'a> Matcher<'a> for ExistsMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("exists")
            .map(|matches| ExistsMatcher { matches })
    }
}
