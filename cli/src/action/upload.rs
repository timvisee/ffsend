use std::path::Path;

use ffsend_api::action::upload::Upload as ApiUpload;
use ffsend_api::reqwest::Client;
use open;

use cmd::cmd_upload::CmdUpload;

/// A file upload action.
pub struct Upload<'a> {
    cmd: &'a CmdUpload<'a>,
}

impl<'a> Upload<'a> {
    /// Construct a new upload action.
    pub fn new(cmd: &'a CmdUpload<'a>) -> Self {
        Self {
            cmd,
        }
    }

    /// Invoke the upload action.
    // TODO: create a trait for this method
    pub fn invoke(&self) {
        // Get API action parameters
        let path = Path::new(self.cmd.file()).to_path_buf();
        let host = self.cmd.host();

        // Create a reqwest client
        let client = Client::new();

        // Execute an upload action
        // TODO: do not unwrap, but return an error
        let file = ApiUpload::new(host, path).invoke(&client).unwrap();

        // Open the URL in the browser
        let url = file.download_url();
        println!("Download URL: {}", url);
        // TODO: do not expect, but return an error
        open::that(url).expect("failed to open URL");
    }
}
