use clap::ArgMatches;
use ffsend_api::action::params::PARAMS_DEFAULT_DOWNLOAD as DOWNLOAD_DEFAULT;
use ffsend_api::api::Version as ApiVersion;
use ffsend_api::url::Url;

use super::Matcher;
use crate::cmd::{
    arg::{ArgDownloadLimit, ArgGenPassphrase, ArgHost, ArgPassword, CmdArgFlag, CmdArgOption},
    matcher::MainMatcher,
};
use crate::util::{bin_name, env_var_present, quit_error_msg, ErrorHintsBuilder};

/// The upload command matcher.
pub struct UploadMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> UploadMatcher<'a> {
    /// Get the selected file to upload.
    // TODO: maybe return a file or path instance here
    pub fn file(&'a self) -> &'a str {
        self.matches
            .value_of("FILE")
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
    /// A generated passphrase will be returned if the user requested so,
    /// otherwise the specified password is returned.
    /// If no password was set, `None` is returned instead.
    ///
    /// The password is returned in the following format:
    /// `(password, generated)`
    pub fn password(&'a self) -> Option<(String, bool)> {
        // Generate a passphrase if requested
        if ArgGenPassphrase::is_present(self.matches) {
            return Some((ArgGenPassphrase::gen_passphrase(), true));
        }

        // Use a specified password or use nothing
        ArgPassword::value(self.matches).map(|password| (password, false))
    }

    /// Get the download limit.
    ///
    /// If the download limit was the default, `None` is returned to not
    /// explicitly set it.
    pub fn download_limit(
        &'a self,
        main_matcher: &MainMatcher,
        api_version: ApiVersion,
        auth: bool,
    ) -> Option<usize> {
        ArgDownloadLimit::value_checked(self.matches, main_matcher, api_version, auth).and_then(
            |d| match d {
                d if d == DOWNLOAD_DEFAULT as usize => None,
                d => Some(d),
            },
        )
    }

    /// Check whether to archive the file to upload.
    #[cfg(feature = "archive")]
    pub fn archive(&self) -> bool {
        self.matches.is_present("archive") || env_var_present("FFSEND_ARCHIVE")
    }

    /// Check whether to open the file URL in the user's browser.
    pub fn open(&self) -> bool {
        self.matches.is_present("open") || env_var_present("FFSEND_OPEN")
    }

    /// Check whether to copy the file URL in the user's clipboard, get the copy mode.
    #[cfg(feature = "clipboard")]
    pub fn copy(&self) -> Option<CopyMode> {
        // Get the options
        let copy = self.matches.is_present("copy") || env_var_present("FFSEND_COPY");
        let copy_cmd = self.matches.is_present("copy-cmd") || env_var_present("FFSEND_COPY_CMD");

        // Return the corresponding copy mode
        if copy_cmd {
            Some(CopyMode::DownloadCmd)
        } else if copy {
            Some(CopyMode::Url)
        } else {
            None
        }
    }

    /// Check whether to shorten a share URL
    #[cfg(feature = "urlshorten")]
    pub fn shorten(&self) -> bool {
        self.matches.is_present("shorten")
    }

    /// Check whether to print a QR code for the share URL.
    #[cfg(feature = "qrcode")]
    pub fn qrcode(&self) -> bool {
        self.matches.is_present("qrcode")
    }
}

impl<'a> Matcher<'a> for UploadMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("upload")
            .map(|matches| UploadMatcher { matches })
    }
}

/// The copy mode.
#[derive(Debug, Copy, Clone)]
pub enum CopyMode {
    /// Copy the public share link.
    Url,

    /// Copy an ffsend download command.
    DownloadCmd,
}

impl CopyMode {
    /// Build the string to copy, based on the given `url` and currend mode.
    pub fn build(&self, url: &str) -> String {
        match self {
            CopyMode::Url => url.into(),
            CopyMode::DownloadCmd => format!("{} download {}", bin_name(), url),
        }
    }
}
