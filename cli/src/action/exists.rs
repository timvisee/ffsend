use clap::ArgMatches;
use ffsend_api::action::exists::{
    Error as ExistsError,
    Exists as ApiExists,
};
use ffsend_api::file::remote_file::{
    FileParseError,
    RemoteFile,
};
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    exists::ExistsMatcher,
};
use error::ActionError;

/// A file exists action.
pub struct Exists<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Exists<'a> {
    /// Construct a new exists action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the exists action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_exists = ExistsMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_exists.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        let file = RemoteFile::parse_url(url, None)?;

        // Make sure the file exists
        let exists_response = ApiExists::new(&file)
            .invoke(&client)?;
        let exists = exists_response.exists();

        // Print the results
        println!("Exists: {:?}", exists);
        if exists {
            println!("Password: {:?}", exists_response.has_password());
        }

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
