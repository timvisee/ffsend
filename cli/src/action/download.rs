use std::path::Path;
use std::sync::{Arc, Mutex};

use ffsend_api::action::download::Download as ApiDownload;
use ffsend_api::file::file::DownloadFile;
use ffsend_api::reqwest::Client;

use cmd::cmd_download::CmdDownload;
use progress::ProgressBar;
use util::open_url;
#[cfg(feature = "clipboard")]
use util::set_clipboard;

/// A file download action.
pub struct Download<'a> {
    cmd: &'a CmdDownload<'a>,
}

impl<'a> Download<'a> {
    /// Construct a new download action.
    pub fn new(cmd: &'a CmdDownload<'a>) -> Self {
        Self {
            cmd,
        }
    }

    /// Invoke the download action.
    // TODO: create a trait for this method
    pub fn invoke(&self) {
        // Get the download URL
        let url = self.cmd.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the file based on the URL
        let file = DownloadFile::parse_url(url)
            .expect("invalid download URL, could not parse file data");

        // Execute an download action
        // TODO: do not unwrap, but return an error
        ApiDownload::new(&file).invoke(&client).unwrap();

        // // Get the download URL, and report it in the console
        // let url = file.download_url(true);
        // println!("Download URL: {}", url);

        // // Open the URL in the browser
        // if self.cmd.open() {
        //     // TODO: do not expect, but return an error
        //     open_url(url.clone()).expect("failed to open URL");
        // }

        // // Copy the URL in the user's clipboard
        // #[cfg(feature = "clipboard")]
        // {
        //     if self.cmd.copy() {
        //         // TODO: do not expect, but return an error
        //         set_clipboard(url.as_str().to_owned())
        //             .expect("failed to put download URL in user clipboard");
        //     }
        // }
        
        panic!("DONE");
    }
}
