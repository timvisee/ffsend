use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use ffsend_api::action::download::{
    Download as ApiDownload,
    Error as DownloadError,
};
use ffsend_api::action::exists::{
    Error as ExistsError,
    Exists as ApiExists,
};
use ffsend_api::file::remote_file::{FileParseError, RemoteFile};
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    download::DownloadMatcher,
};
use progress::ProgressBar;
use util::ensure_password;

/// A file download action.
pub struct Download<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Download<'a> {
    /// Construct a new download action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the download action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), Error> {
        // Create the command matchers
        let matcher_download = DownloadMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_download.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        // TODO: handle error here
        let file = RemoteFile::parse_url(url, None)?;

        // Get the target file or directory, and the password
        let target = matcher_download.output();
        let mut password = matcher_download.password();

        // Check whether the file exists
        let exists = ApiExists::new(&file).invoke(&client)?;
        if !exists.exists() {
            return Err(Error::Expired);
        }

        // Ensure a password is set when required
        ensure_password(&mut password, exists.has_password());

        // Create a progress bar reporter
        let bar = Arc::new(Mutex::new(ProgressBar::new_download()));

        // Execute an download action
        ApiDownload::new(
            &file,
            target,
            password,
            false,
        ).invoke(&client, bar)?;

        // TODO: open the file, or it's location
        // TODO: copy the file location

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "Invalid share URL")]
    InvalidUrl(#[cause] FileParseError),

    /// An error occurred while checking if the file exists.
    #[fail(display = "Failed to check whether the file exists")]
    Exists(#[cause] ExistsError),

    /// An error occurred while downloading the file.
    #[fail(display = "")]
    Download(#[cause] DownloadError),

    /// The given Send file has expired, or did never exist in the first place.
    #[fail(display = "The file has expired or did never exist")]
    Expired,
}

impl From<FileParseError> for Error {
    fn from(err: FileParseError) -> Error {
        Error::InvalidUrl(err)
    }
}

impl From<ExistsError> for Error {
    fn from(err: ExistsError) -> Error {
        Error::Exists(err)
    }
}

impl From<DownloadError> for Error {
    fn from(err: DownloadError) -> Error {
        Error::Download(err)
    }
}
