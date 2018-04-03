use clap::ArgMatches;
use ffsend_api::action::password::Password as ApiPassword;
use ffsend_api::file::remote_file::RemoteFile;
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    password::PasswordMatcher,
};
use error::ActionError;
use util::print_success;

/// A file password action.
pub struct Password<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Password<'a> {
    /// Construct a new password action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the password action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_password = PasswordMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_password.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        // TODO: handle error here
        let file = RemoteFile::parse_url(url, matcher_password.owner())?;

        // TODO: show an informative error if the owner token isn't set

        // Execute an password action
        ApiPassword::new(&file, &matcher_password.password(), None).invoke(&client)?;

        // Print a success message
        print_success("Password set");

        Ok(())
    }
}
