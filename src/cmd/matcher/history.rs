use clap::ArgMatches;
use failure::Fail;
use ffsend_api::url::Url;

use super::Matcher;
use crate::host::parse_host;
use crate::util::{quit_error, ErrorHints};

/// The history command matcher.
pub struct HistoryMatcher<'a> {
    #[allow(unused)]
    matches: &'a ArgMatches<'a>,
}

impl<'a> HistoryMatcher<'a> {
    /// Check whether to clear all history.
    pub fn clear(&self) -> bool {
        self.matches.is_present("clear")
    }

    /// Check whether to remove a given entry from the history.
    ///
    /// This method parses the URL into an `Url`.
    /// If the given URL is invalid,
    /// the program will quit with an error message.
    pub fn rm(&'a self) -> Option<Url> {
        // Get the URL
        let url = self.matches.value_of("rm")?;

        // Parse the URL
        match parse_host(&url) {
            Ok(url) => Some(url),
            Err(err) => quit_error(
                err.context("failed to parse the given share URL"),
                ErrorHints::default(),
            ),
        }
    }
}

impl<'a> Matcher<'a> for HistoryMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("history")
            .map(|matches| HistoryMatcher { matches })
    }
}
