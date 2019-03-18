pub mod completions;

use clap::ArgMatches;

use crate::cmd::matcher::{generate::GenerateMatcher, Matcher};
use crate::error::ActionError;
use completions::Completions;

/// A file generate action.
pub struct Generate<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Generate<'a> {
    /// Construct a new generate action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the generate action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matcher
        let matcher_generate = GenerateMatcher::with(self.cmd_matches).unwrap();

        // Match shell completions
        if matcher_generate.matcher_completions().is_some() {
            return Completions::new(self.cmd_matches).invoke();
        }

        // Unreachable, clap will print help for missing sub command instead
        unreachable!()
    }
}
