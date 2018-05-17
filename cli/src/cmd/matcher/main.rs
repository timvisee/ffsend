#[cfg(feature = "history")]
use std::path::PathBuf;

use clap::ArgMatches;

use super::Matcher;
use util::env_var_present;
#[cfg(feature = "history")]
use util::{ErrorHintsBuilder, quit_error_msg};

/// The main command matcher.
pub struct MainMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> MainMatcher<'a> {
    /// Check whether to force.
    pub fn force(&self) -> bool {
        self.matches.is_present("force") || env_var_present("FFSEND_FORCE")
    }

    /// Check whether to use no-interact mode.
    pub fn no_interact(&self) -> bool {
        self.matches.is_present("no-interact") || env_var_present("FFSEND_NO_INTERACT")
    }

    /// Check whether to assume yes.
    pub fn assume_yes(&self) -> bool {
        self.matches.is_present("yes") || env_var_present("FFSEND_YES")
    }

    /// Get the history file to use.
    #[cfg(feature = "history")]
    pub fn history(&self) -> PathBuf {
        // Get the path
        let path = self.matches.value_of("history")
            .map(PathBuf::from);

        // Ensure the path is correct
        match path {
            Some(path) => path,
            None => quit_error_msg(
                "history file path not set",
                ErrorHintsBuilder::default()
                    .history(true)
                    .verbose(false)
                    .build()
                    .unwrap(),
            ),
        }
    }

    /// Check whether we are incognito from the file history.
    #[cfg(feature = "history")]
    pub fn incognito(&self) -> bool {
        self.matches.is_present("incognito") || env_var_present("FFSEND_INCOGNITO")
    }

    /// Check whether verbose mode is used.
    pub fn verbose(&self) -> bool {
        self.matches.is_present("verbose") || env_var_present("FFSEND_VERBOSE")
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
