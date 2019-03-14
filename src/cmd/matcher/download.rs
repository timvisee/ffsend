use std::path::PathBuf;

use clap::ArgMatches;
use ffsend_api::url::Url;

use super::Matcher;
use crate::cmd::arg::{ArgPassword, ArgUrl, CmdArgOption};
#[cfg(feature = "archive")]
use crate::util::env_var_present;

/// The download command matcher.
pub struct DownloadMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> DownloadMatcher<'a> {
    /// Get the file share URL.
    ///
    /// This method parses the URL into an `Url`.
    /// If the given URL is invalid,
    /// the program will quit with an error message.
    pub fn url(&'a self) -> Url {
        ArgUrl::value(self.matches)
    }

    /// Guess the file share host, based on the file share URL.
    ///
    /// See `Self::url`.
    pub fn guess_host(&'a self, url: Option<Url>) -> Url {
        let mut url = url.unwrap_or(self.url());
        url.set_path("");
        url.set_query(None);
        url.set_fragment(None);
        url
    }

    /// Get the password.
    /// `None` is returned if no password was specified.
    pub fn password(&'a self) -> Option<String> {
        ArgPassword::value(self.matches)
    }

    /// The target file or directory to download the file to.
    /// If a directory is given, the file name of the original uploaded file
    /// will be used.
    pub fn output(&'a self) -> PathBuf {
        self.matches
            .value_of("output")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("./"))
    }

    /// Check whether to extract an archived file.
    #[cfg(feature = "archive")]
    pub fn extract(&self) -> bool {
        self.matches.is_present("extract") || env_var_present("FFSEND_EXTRACT")
    }
}

impl<'a> Matcher<'a> for DownloadMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("download")
            .map(|matches| DownloadMatcher { matches })
    }
}
