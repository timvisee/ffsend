pub mod completions;

use clap::ArgMatches;

use super::Matcher;
use completions::CompletionsMatcher;

/// The generate command matcher.
pub struct GenerateMatcher<'a> {
    root: &'a ArgMatches<'a>,
    _matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> GenerateMatcher<'a> {
    /// Get the generate completions sub command, if matched.
    pub fn matcher_completions(&'a self) -> Option<CompletionsMatcher> {
        CompletionsMatcher::with(&self.root)
    }
}

impl<'a> Matcher<'a> for GenerateMatcher<'a> {
    fn with(root: &'a ArgMatches) -> Option<Self> {
        root.subcommand_matches("generate")
            .map(|matches| GenerateMatcher {
                root,
                _matches: matches,
            })
    }
}
