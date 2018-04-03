use clap::ArgMatches;
use ffsend_api::action::delete::{
    Error as DeleteError,
    Delete as ApiDelete,
};
use ffsend_api::file::remote_file::{
    FileParseError,
    RemoteFile,
};
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    delete::DeleteMatcher,
};
use error::ActionError;
use util::print_success;

/// A file delete action.
pub struct Delete<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Delete<'a> {
    /// Construct a new delete action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the delete action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_delete = DeleteMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_delete.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL, get the password
        let file = RemoteFile::parse_url(url, matcher_delete.owner())?;

        // TODO: show an informative error if the owner token isn't set

        // Send the file deletion request
        ApiDelete::new(&file, None).invoke(&client)?;

        // Print a success message
        print_success("File deleted");

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "Invalid share URL")]
    InvalidUrl(#[cause] FileParseError),

    /// An error occurred while deleting the remote file.
    #[fail(display = "Failed to delete the shared file")]
    Delete(#[cause] DeleteError),
}

impl From<FileParseError> for Error {
    fn from(err: FileParseError) -> Error {
        Error::InvalidUrl(err)
    }
}

impl From<DeleteError> for Error {
    fn from(err: DeleteError) -> Error {
        Error::Delete(err)
    }
}
