use clap::ArgMatches;

use super::Matcher;

/// The history command matcher.
pub struct HistoryMatcher<'a> {
    #[allow(unused)]
    matches: &'a ArgMatches<'a>,
}

impl<'a> Matcher<'a> for HistoryMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("history")
            .map(|matches| HistoryMatcher { matches })
    }
}
