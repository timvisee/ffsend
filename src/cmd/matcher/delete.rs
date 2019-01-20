use clap::ArgMatches;
use ffsend_api::url::Url;

use super::Matcher;
use crate::cmd::arg::{ArgOwner, ArgUrl, CmdArgOption};

/// The delete command matcher.
pub struct DeleteMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> DeleteMatcher<'a> {
    /// Get the file share URL.
    ///
    /// This method parses the URL into an `Url`.
    /// If the given URL is invalid,
    /// the program will quit with an error message.
    pub fn url(&'a self) -> Url {
        ArgUrl::value(self.matches)
    }

    /// Get the owner token.
    pub fn owner(&'a self) -> Option<String> {
        // TODO: just return a string reference here?
        ArgOwner::value(self.matches).map(|token| token.to_owned())
    }
}

impl<'a> Matcher<'a> for DeleteMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("delete")
            .map(|matches| DeleteMatcher { matches })
    }
}
