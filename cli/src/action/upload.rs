use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use failure::{err_msg, Fail};
use ffsend_api::action::params::ParamsDataBuilder;
use ffsend_api::action::upload::Upload as ApiUpload;
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    upload::UploadMatcher,
};
use error::ActionError;
use progress::ProgressBar;
use util::open_url;
#[cfg(feature = "clipboard")]
use util::{print_error, set_clipboard};

/// A file upload action.
pub struct Upload<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Upload<'a> {
    /// Construct a new upload action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the upload action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_upload = UploadMatcher::with(self.cmd_matches).unwrap();

        // Get API parameters
        let path = Path::new(matcher_upload.file()).to_path_buf();
        let host = matcher_upload.host();

        // Create a reqwest client
        let client = Client::new();

        // Create a progress bar reporter
        let bar = Arc::new(Mutex::new(ProgressBar::new_upload()));

        // Build a parameters object to set for the file
        let params = {
            // Build the parameters data object
            let mut params = ParamsDataBuilder::default()
                .download_limit(matcher_upload.download_limit())
                .build()
                .unwrap();

            // Wrap the data in an option if not empty
            if params.is_empty() {
                None
            } else {
                Some(params)
            }
        };

        // Execute an upload action
        let file = ApiUpload::new(
            host,
            path,
            matcher_upload.name().map(|name| name.to_owned()),
            matcher_upload.password(),
            params,
        ).invoke(&client, bar)?;

        // Get the download URL, and report it in the console
        let url = file.download_url(true);
        println!("Download URL: {}", url);
        println!("Owner token: {}", file.owner_token().unwrap());

        // Open the URL in the browser
        if matcher_upload.open() {
            if let Err(err) = open_url(url.clone()) {
                print_error(
                    err.context("Failed to open the URL in the browser")
                );
            };
        }

        // Copy the URL in the user's clipboard
        #[cfg(feature = "clipboard")]
        {
            if matcher_upload.copy() {
                if set_clipboard(url.as_str().to_owned()).is_err() {
                    print_error(
                        err_msg("Failed to copy the URL to the clipboard")
                            .compat()
                    );
                }
            }
        }

        Ok(())
    }
}
