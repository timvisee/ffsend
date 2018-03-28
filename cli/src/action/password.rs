use ffsend_api::action::password::Password as ApiPassword;
use ffsend_api::file::file::DownloadFile;
use ffsend_api::reqwest::Client;

use cmd::cmd_password::CmdPassword;
use error::ActionError;
use util::print_success;

/// A file password action.
pub struct Password<'a> {
    cmd: &'a CmdPassword<'a>,
}

impl<'a> Password<'a> {
    /// Construct a new password action.
    pub fn new(cmd: &'a CmdPassword<'a>) -> Self {
        Self {
            cmd,
        }
    }

    /// Invoke the password action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Get the share URL
        let url = self.cmd.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the file based on the URL
        // TODO: handle error here
        let file = DownloadFile::parse_url(url)
            .expect("invalid share URL, could not parse file data");

        // Execute an password action
        ApiPassword::new(&file, &self.cmd.password()).invoke(&client)?;

        // Print a success message
        print_success("Password set");

        Ok(())
    }
}
