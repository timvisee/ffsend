use std::path::PathBuf;

use clap::ArgMatches;
use ffsend_api::url::Url;

use cmd::arg::{ArgPassword, ArgUrl, CmdArgOption};
use super::Matcher;

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

    /// Get the password.
    /// `None` is returned if no password was specified.
    pub fn password(&'a self) -> Option<String> {
        ArgPassword::value(self.matches)
    }

    /// The target file or directory to download the file to.
    /// If a directory is given, the file name of the original uploaded file
    /// will be used.
    pub fn output(&'a self) -> PathBuf {
        self.matches.value_of("output")
            .map(|path| PathBuf::from(path))
            .unwrap_or(PathBuf::from("./"))
    }
}

impl<'a> Matcher<'a> for DownloadMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches.subcommand_matches("download")
            .map(|matches|
                 DownloadMatcher {
                     matches,
                 }
            )
    }
}
