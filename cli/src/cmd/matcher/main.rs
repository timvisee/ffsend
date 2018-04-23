use std::path::PathBuf;

use clap::ArgMatches;

use super::Matcher;
use util::{ErrorHintsBuilder, quit_error_msg};

/// The main command matcher.
pub struct MainMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> MainMatcher<'a> {
    /// Check whether to force.
    pub fn force(&self) -> bool {
        self.matches.is_present("force")
    }

    /// Check whether to use no-interact mode.
    pub fn no_interact(&self) -> bool {
        self.matches.is_present("no-interact")
    }

    /// Check whether to assume yes.
    pub fn assume_yes(&self) -> bool {
        self.matches.is_present("yes")
    }

    /// Get the history file to use.
    pub fn history(&self) -> PathBuf {
        // Get the path
        let path = self.matches.value_of("history")
            .map(|path| PathBuf::from(path));

        // Ensure the path is correct
        match path {
            Some(path) => path,
            None => quit_error_msg(
                "History file path not set",
                ErrorHintsBuilder::default()
                    .history(true)
                    .verbose(false)
                    .build()
                    .unwrap(),
            ),
        }
    }

    /// Check whether we are incognito from the file history.
    pub fn incognito(&self) -> bool {
        self.matches.is_present("incognito")
    }
}

impl<'a> Matcher<'a> for MainMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        Some(
            MainMatcher {
                matches,
            }
        )
    }
}
