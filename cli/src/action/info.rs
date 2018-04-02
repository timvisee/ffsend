use ffsend_api::action::info::Info as ApiInfo;
use ffsend_api::file::remote_file::RemoteFile;
use ffsend_api::reqwest::Client;

use cmd::cmd_info::CmdInfo;
use error::ActionError;

/// A file info action.
pub struct Info<'a> {
    cmd: &'a CmdInfo<'a>,
}

impl<'a> Info<'a> {
    /// Construct a new info action.
    pub fn new(cmd: &'a CmdInfo<'a>) -> Self {
        Self {
            cmd,
        }
    }

    /// Invoke the info action.
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

        // Execute the info fetch action
        let info = ApiInfo::new(&file, None).invoke(&client)?;

        println!("{:#?}", info);

        Ok(())
    }
}
