use ffsend_api::action::params::{
    Params as ApiParams,
    ParamsDataBuilder,
};
use ffsend_api::file::remote_file::RemoteFile;
use ffsend_api::reqwest::Client;

use cmd::cmd_params::CmdParams;
use error::ActionError;
use util::print_success;

/// A file parameters action.
pub struct Params<'a> {
    cmd: &'a CmdParams<'a>,
}

impl<'a> Params<'a> {
    /// Construct a new parameters action.
    pub fn new(cmd: &'a CmdParams<'a>) -> Self {
        Self {
            cmd,
        }
    }

    /// Invoke the parameters action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Get the share URL
        let url = self.cmd.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        // TODO: handle error here
        let file = RemoteFile::parse_url(url, self.cmd.owner())?;

        // TODO: show an informative error if the owner token isn't set

        // Build the parameters data object
        let data = ParamsDataBuilder::default()
            .download_limit(self.cmd.download_limit())
            .build()
            .unwrap();

        // TODO: make sure the data isn't empty

        // Execute an password action
        ApiParams::new(&file, data, None).invoke(&client)?;

        // Print a success message
        print_success("Parameters set");

        Ok(())
    }
}
