use clap::ArgMatches;

use super::Matcher;

/// The debug command matcher.
pub struct DebugMatcher<'a> {
    #[allow(dead_code)]
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
