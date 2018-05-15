use clap::ArgMatches;
use ffsend_api::action::params::{
    PARAMS_DEFAULT_DOWNLOAD as DOWNLOAD_DEFAULT,
};
use ffsend_api::url::Url;

use cmd::arg::{ArgDownloadLimit, ArgHost, ArgPassword, CmdArgOption};
use super::Matcher;
use util::{ErrorHintsBuilder, quit_error_msg};

/// The upload command matcher.
pub struct UploadMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> UploadMatcher<'a> {
    /// Get the selected file to upload.
    // TODO: maybe return a file or path instance here
    pub fn file(&'a self) -> &'a str {
        self.matches.value_of("FILE")
            .expect("no file specified to upload")
    }

    /// The the name to use for the uploaded file.
    /// If no custom name is given, none is returned.
    // TODO: validate custom names, no path separators
    // TODO: only allow extension renaming with force flag
    pub fn name(&'a self) -> Option<&'a str> {
        // Get the chosen file name
        let name = self.matches.value_of("name")?;

        // The file name must not be empty
        // TODO: allow to force an empty name here, and process emtpy names on downloading
        if name.trim().is_empty() {
            quit_error_msg(
                "the file name must not be empty",
                ErrorHintsBuilder::default()
                    .force(false)
                    .verbose(false)
                    .build()
                    .unwrap(),
            );
        }

        Some(name)
    }

    /// Get the host to upload to.
    ///
    /// This method parses the host into an `Url`.
    /// If the given host is invalid,
    /// the program will quit with an error message.
    pub fn host(&'a self) -> Url {
        ArgHost::value(self.matches)
    }

    /// Get the password.
    /// `None` is returned if no password was specified.
    pub fn password(&'a self) -> Option<String> {
        ArgPassword::value(self.matches)
    }

    /// Get the download limit.
    /// If the download limit was the default, `None` is returned to not
    /// explicitly set it.
    pub fn download_limit(&'a self) -> Option<u8> {
        ArgDownloadLimit::value(self.matches)
            .and_then(|d| match d {
                DOWNLOAD_DEFAULT => None,
                d => Some(d),
            })
    }

    /// Check whether to archive the file to upload.
    /// TODO: infer to use this flag if a directory is selected
    pub fn archive(&self) -> bool {
        self.matches.is_present("archive")
    }

    /// Check whether to open the file URL in the user's browser.
    pub fn open(&self) -> bool {
        self.matches.is_present("open")
    }

    /// Check whether to copy the file URL in the user's clipboard.
    #[cfg(feature = "clipboard")]
    pub fn copy(&self) -> bool {
        self.matches.is_present("copy")
    }
}

impl<'a> Matcher<'a> for UploadMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches.subcommand_matches("upload")
            .map(|matches|
                 UploadMatcher {
                     matches,
                 }
            )
    }
}
