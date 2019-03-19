use clap::ArgMatches;
use ffsend_api::action::password::{Error as PasswordError, Password as ApiPassword};
use ffsend_api::file::remote_file::RemoteFile;
use prettytable::{format::FormatBuilder, Cell, Row, Table};

use crate::client::create_config;
use crate::cmd::matcher::{main::MainMatcher, password::PasswordMatcher, Matcher};
use crate::error::ActionError;
#[cfg(feature = "history")]
use crate::history_tool;
use crate::util::{ensure_owner_token, print_success};

/// A file password action.
pub struct Password<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Password<'a> {
    /// Construct a new password action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
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
        let client_config = create_config(&matcher_main);
        let client = client_config.client(false);

        // Parse the remote file based on the share URL, derive the owner token from history
        let mut file = RemoteFile::parse_url(url, matcher_password.owner())?;
        #[cfg(feature = "history")]
        history_tool::derive_file_properties(&matcher_main, &mut file);

        // Ensure the owner token is set
        ensure_owner_token(file.owner_token_mut(), &matcher_main, false);

        // Get the password to use and whether it was generated
        let (password, password_generated) = matcher_password.password();

        // Execute an password action
        let result = ApiPassword::new(&file, &password, None).invoke(&client);
        if let Err(PasswordError::Expired) = result {
            // Remove the file from the history if expired
            #[cfg(feature = "history")]
            history_tool::remove(&matcher_main, &file);
        }
        result?;

        // Add the file to the history
        #[cfg(feature = "history")]
        history_tool::add(&matcher_main, file, true);

        // Print the passphrase if one was generated
        if password_generated {
            let mut table = Table::new();
            table.set_format(FormatBuilder::new().padding(0, 2).build());
            table.add_row(Row::new(vec![
                Cell::new("Passphrase:"),
                Cell::new(&password),
            ]));
            table.printstd();
        }

        // Print a success message
        print_success("Password set");

        Ok(())
    }
}
