use clap::ArgMatches;
use ffsend_api::url::Url;

use super::Matcher;
use cmd::arg::{ArgDownloadLimit, ArgOwner, ArgUrl, CmdArgOption};

/// The params command matcher.
pub struct ParamsMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> ParamsMatcher<'a> {
    /// Get the file share URL.
    ///
    /// This method parses the URL into an `Url`.
    /// If the given URL is invalid,
    /// the program will quit with an error message.
    pub fn url(&'a self) -> Url {
        ArgUrl::value(self.matches)
    }

    /// Get the owner token.
    pub fn owner(&'a self) -> Option<String> {
        // TODO: just return a string reference here?
        ArgOwner::value(self.matches).map(|token| token.to_owned())
    }

    /// Get the download limit.
    pub fn download_limit(&'a self) -> Option<u8> {
        ArgDownloadLimit::value(self.matches)
    }
}

impl<'a> Matcher<'a> for ParamsMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("parameters")
            .map(|matches| ParamsMatcher { matches })
    }
}
