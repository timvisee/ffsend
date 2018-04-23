use clap::ArgMatches;
use ffsend_api::action::password::{
    Error as PasswordError,
    Password as ApiPassword,
};
use ffsend_api::file::remote_file::RemoteFile;
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    main::MainMatcher,
    password::PasswordMatcher,
};
use error::ActionError;
use history_tool;
use util::{ensure_owner_token, print_success};

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
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_password = PasswordMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_password.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        let mut file = RemoteFile::parse_url(url, matcher_password.owner())?;

        // Ensure the owner token is set
        ensure_owner_token(file.owner_token_mut(), &matcher_main);

        // Execute an password action
        let result = ApiPassword::new(
            &file,
            &matcher_password.password(),
            None,
        ).invoke(&client);
        if let Err(PasswordError::Expired) = result {
            // Remove the file from the history if expired
            history_tool::remove(&matcher_main, &file);
        }
        result?;

        // Add the file to the history
        history_tool::add(&matcher_main, file);

        // Print a success message
        print_success("Password set");

        Ok(())
    }
}
