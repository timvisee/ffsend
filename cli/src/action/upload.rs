use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use failure::{err_msg, Fail};
use ffsend_api::action::params::ParamsDataBuilder;
use ffsend_api::action::upload::Upload as ApiUpload;
use ffsend_api::config::{UPLOAD_SIZE_MAX, UPLOAD_SIZE_MAX_RECOMMENDED};
use ffsend_api::reqwest::Client;

use cmd::matcher::{Matcher, MainMatcher, UploadMatcher};
use error::ActionError;
use history::History;
use progress::ProgressBar;
use util::{
    ErrorHintsBuilder,
    format_bytes,
    open_url,
    print_error,
    print_error_msg,
    prompt_yes,
    quit,
    quit_error_msg,
};
#[cfg(feature = "clipboard")]
use util::set_clipboard;

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
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_upload = UploadMatcher::with(self.cmd_matches).unwrap();

        // Get API parameters
        let path = Path::new(matcher_upload.file()).to_path_buf();
        let host = matcher_upload.host();

        // TODO: ensure the file exists and is accessible

        // Get the file size to warn about large files
        if let Ok(size) = File::open(&path)
            .and_then(|f| f.metadata())
            .map(|m| m.len())
        {
            if size > UPLOAD_SIZE_MAX && !matcher_main.force() {
                // The file is too large, show an error and quit
                quit_error_msg(
                    format!(
                        "The file size is {}, bigger than the maximum allowed of {}",
                        format_bytes(size),
                        format_bytes(UPLOAD_SIZE_MAX),
                    ),
                    ErrorHintsBuilder::default()
                        .force(true)
                        .verbose(false)
                        .build()
                        .unwrap(),
                );
            } else if size > UPLOAD_SIZE_MAX_RECOMMENDED && !matcher_main.force() {
                // The file is larger than the recommended maximum, warn
                eprintln!(
                    "The file size is {}, bigger than the recommended maximum of {}",
                    format_bytes(size),
                    format_bytes(UPLOAD_SIZE_MAX_RECOMMENDED),
                );

                // Prompt the user to continue, quit if the user answered no
                if !prompt_yes("Continue uploading?", Some(true), &matcher_main) {
                    println!("Upload cancelled");
                    quit();
                }
            }
        } else {
            print_error_msg("Failed to check the file size, ignoring");
        }

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

        // Add the file to the history manager
        // TODO: specify the proper path here
        let history_path = PathBuf::from("./history.toml");
        if let Err(err) = History::load_add_save(history_path, file.clone()) {
            print_error(err.context("Failed to add file to history, ignoring"));
        }

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
