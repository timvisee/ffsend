use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use ffsend_api::action::download::Download as ApiDownload;
use ffsend_api::action::exists::Exists as ApiExists;
use ffsend_api::file::remote_file::RemoteFile;
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    download::DownloadMatcher,
};
use error::ActionError;
use progress::ProgressBar;
use util::prompt_password;

/// A file download action.
pub struct Download<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Download<'a> {
    /// Construct a new download action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the download action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_download = DownloadMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_download.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        // TODO: handle error here
        let file = RemoteFile::parse_url(url, None)?;

        // Get the target file or directory, and the password
        let target = matcher_download.output();
        let mut password = matcher_download.password();

        // Check whether the file requires a password
        let exists = ApiExists::new(&file).invoke(&client).unwrap();
        if exists.has_password() != password.is_some() {
            if exists.has_password() {
                println!("This file is protected with a password.");
                password = Some(prompt_password());
            } else {
                println!("Ignoring password, it is not required");
                password = None;
            }
        }

        // Create a progress bar reporter
        let bar = Arc::new(Mutex::new(ProgressBar::new_download()));

        // Execute an download action
        ApiDownload::new(
            &file,
            target,
            password,
            false,
        ).invoke(&client, bar)?;

        // TODO: open the file, or it's location
        // TODO: copy the file location

        Ok(())
    }
}
