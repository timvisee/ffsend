use clap::ArgMatches;
use ffsend_api::action::params::{Error as ParamsError, Params as ApiParams, ParamsDataBuilder};
use ffsend_api::file::remote_file::RemoteFile;

use client::create_client;
use cmd::matcher::{main::MainMatcher, params::ParamsMatcher, Matcher};
use error::ActionError;
#[cfg(feature = "history")]
use history_tool;
use util::{ensure_owner_token, print_success};

/// A file parameters action.
pub struct Params<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Params<'a> {
    /// Construct a new parameters action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the parameters action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_params = ParamsMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_params.url();

        // Create a reqwest client
        let client = create_client();

        // Parse the remote file based on the share URL, derive the owner token from history
        let mut file = RemoteFile::parse_url(url, matcher_params.owner())?;
        #[cfg(feature = "history")]
        history_tool::derive_file_properties(&matcher_main, &mut file);

        // Ensure the owner token is set
        ensure_owner_token(file.owner_token_mut(), &matcher_main);

        // Build the parameters data object
        let data = ParamsDataBuilder::default()
            .download_limit(matcher_params.download_limit())
            .build()
            .unwrap();

        // TODO: make sure the data isn't empty

        // Execute an password action
        let result = ApiParams::new(&file, data, None).invoke(&client);
        if let Err(ParamsError::Expired) = result {
            // Remove the file from the history if expired
            #[cfg(feature = "history")]
            history_tool::remove(&matcher_main, &file);
        }
        result?;

        // Update the history
        #[cfg(feature = "history")]
        history_tool::add(&matcher_main, file, true);

        // Print a success message
        print_success("Parameters set");

        Ok(())
    }
}
