#[cfg(feature = "history")]
use std::path::PathBuf;

use clap::ArgMatches;
use ffsend_api::api::DesiredVersion;

use super::Matcher;
use crate::cmd::arg::{ArgApi, ArgBasicAuth, CmdArgOption};
use crate::util::{env_var_present, parse_duration};
#[cfg(feature = "history")]
use crate::util::{quit_error_msg, ErrorHintsBuilder};

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

    /// Get the desired API version to use.
    pub fn api(&'a self) -> DesiredVersion {
        ArgApi::value(self.matches)
    }

    /// Get basic HTTP authentication credentials to use.
    pub fn basic_auth(&'a self) -> Option<(String, Option<String>)> {
        ArgBasicAuth::value(self.matches)
    }

    /// Get the history file to use.
    #[cfg(feature = "history")]
    pub fn history(&self) -> PathBuf {
        // Get the path
        let path = self.matches.value_of("history").map(PathBuf::from);

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

    /// Get the timeout in seconds
    pub fn timeout(&self) -> u64 {
        self.matches
            .value_of("timeout")
            .and_then(|arg| parse_duration(arg).ok())
            .expect("invalid timeout value") as u64
    }

    /// Get the transfer timeout in seconds
    pub fn transfer_timeout(&self) -> u64 {
        self.matches
            .value_of("transfer-timeout")
            .and_then(|arg| parse_duration(arg).ok())
            .expect("invalid transfer-timeout value") as u64
    }

    /// Check whether we are incognito from the file history.
    #[cfg(feature = "history")]
    pub fn incognito(&self) -> bool {
        self.matches.is_present("incognito") || env_var_present("FFSEND_INCOGNITO")
    }

    /// Check whether quiet mode is used.
    pub fn quiet(&self) -> bool {
        !self.verbose() && (self.matches.is_present("quiet") || env_var_present("FFSEND_QUIET"))
    }

    /// Check whether verbose mode is used.
    pub fn verbose(&self) -> bool {
        self.matches.is_present("verbose") || env_var_present("FFSEND_VERBOSE")
    }
}

impl<'a> Matcher<'a> for MainMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        Some(MainMatcher { matches })
    }
}
