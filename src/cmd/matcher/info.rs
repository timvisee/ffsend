use ffsend_api::url::Url;

use clap::ArgMatches;

use super::Matcher;
use crate::cmd::arg::{ArgOwner, ArgPassword, ArgUrl, CmdArgOption};

/// The info command matcher.
pub struct InfoMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> InfoMatcher<'a> {
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

    /// Get the password.
    /// `None` is returned if no password was specified.
    pub fn password(&'a self) -> Option<String> {
        ArgPassword::value(self.matches)
    }
}

impl<'a> Matcher<'a> for InfoMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("info")
            .map(|matches| InfoMatcher { matches })
    }
}
