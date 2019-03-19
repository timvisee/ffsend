use clap::ArgMatches;
use ffsend_api::action::delete::{Delete as ApiDelete, Error as DeleteError};
use ffsend_api::file::remote_file::{FileParseError, RemoteFile};

use crate::client::create_config;
use crate::cmd::matcher::{delete::DeleteMatcher, main::MainMatcher, Matcher};
use crate::error::ActionError;
#[cfg(feature = "history")]
use crate::history_tool;
use crate::util::{ensure_owner_token, print_success};

/// A file delete action.
pub struct Delete<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Delete<'a> {
    /// Construct a new delete action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the delete action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_delete = DeleteMatcher::with(self.cmd_matches).unwrap();

        // Get the share link
        let url = matcher_delete.url();

        // Create client
        let client_config = create_config(&matcher_main);
        let client = client_config.client(false);

        // Parse the remote file based on the share link, derive the owner token from history
        let mut file = RemoteFile::parse_url(url, matcher_delete.owner())?;
        #[cfg(feature = "history")]
        history_tool::derive_file_properties(&matcher_main, &mut file);

        // Ensure the owner token is set
        ensure_owner_token(file.owner_token_mut(), &matcher_main, false);

        // Send the file deletion request
        let result = ApiDelete::new(&file, None).invoke(&client);
        if let Err(DeleteError::Expired) = result {
            // Remove the file from the history manager if it does not exist
            #[cfg(feature = "history")]
            history_tool::remove(&matcher_main, &file);
        }
        result?;

        // Remove the file from the history manager
        #[cfg(feature = "history")]
        history_tool::remove(&matcher_main, &file);

        // Print a success message
        print_success("File deleted");

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "invalid share link")]
    InvalidUrl(#[cause] FileParseError),

    /// Could not delete, the file has expired or did never exist.
    #[fail(display = "the file has expired or did never exist")]
    Expired,

    /// An error occurred while deleting the remote file.
    #[fail(display = "failed to delete the shared file")]
    Delete(#[cause] DeleteError),
}

impl From<FileParseError> for Error {
    fn from(err: FileParseError) -> Error {
        Error::InvalidUrl(err)
    }
}

impl From<DeleteError> for Error {
    fn from(err: DeleteError) -> Error {
        match err {
            DeleteError::Expired => Error::Expired,
            err => Error::Delete(err),
        }
    }
}
