use clap::ArgMatches;
use ffsend_api::action::exists::{Error as ExistsError, Exists as ApiExists};
use ffsend_api::file::remote_file::{FileParseError, RemoteFile};

use crate::client::create_config;
use crate::cmd::matcher::main::MainMatcher;
use crate::cmd::matcher::{exists::ExistsMatcher, Matcher};
use crate::error::ActionError;
#[cfg(feature = "history")]
use crate::history_tool;

/// A file exists action.
pub struct Exists<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Exists<'a> {
    /// Construct a new exists action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the exists action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_exists = ExistsMatcher::with(self.cmd_matches).unwrap();
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_exists.url();

        // Create a reqwest client
        let client_config = create_config(&matcher_main);
        let client = client_config.client(false);

        // Parse the remote file based on the share URL
        let file = RemoteFile::parse_url(url, None)?;

        // Make sure the file exists
        let exists_response = ApiExists::new(&file).invoke(&client)?;
        let exists = exists_response.exists();

        // Print the results
        println!("Exists: {:?}", exists);
        if exists {
            println!("Password: {:?}", exists_response.requires_password());
        }

        // Add or remove the file from the history
        #[cfg(feature = "history")]
        {
            if exists {
                history_tool::add(&matcher_main, file, false);
            } else {
                history_tool::remove(&matcher_main, &file);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "invalid share link")]
    InvalidUrl(#[cause] FileParseError),

    /// An error occurred while checking if the file exists.
    #[fail(display = "failed to check whether the file exists")]
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
