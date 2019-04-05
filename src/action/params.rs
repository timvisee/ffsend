use clap::ArgMatches;
use ffsend_api::action::params::{Error as ParamsError, Params as ApiParams, ParamsDataBuilder};
use ffsend_api::file::remote_file::RemoteFile;

use super::select_api_version;
use crate::client::create_config;
use crate::cmd::matcher::{main::MainMatcher, params::ParamsMatcher, Matcher};
use crate::error::ActionError;
#[cfg(feature = "history")]
use crate::history_tool;
use crate::util::{ensure_owner_token, print_success};

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

        // Get the share URL and the host
        // TODO: derive host through helper function
        let url = matcher_params.url();
        let mut host = url.clone();
        host.set_path("");
        host.set_query(None);
        host.set_fragment(None);

        // Create a reqwest client
        let client_config = create_config(&matcher_main);
        let client = client_config.client(false);

        // Determine the API version to use
        let mut desired_version = matcher_main.api();
        select_api_version(&client, host, &mut desired_version)?;
        let api_version = desired_version.version().unwrap();

        // Parse the remote file based on the share URL, derive the owner token from history
        let mut file = RemoteFile::parse_url(url, matcher_params.owner())?;
        #[cfg(feature = "history")]
        history_tool::derive_file_properties(&matcher_main, &mut file);

        // Ensure the owner token is set
        ensure_owner_token(file.owner_token_mut(), &matcher_main, false);

        // We don't authenticate for now
        let auth = false;

        // Build the parameters data object
        let data = ParamsDataBuilder::default()
            .download_limit(
                matcher_params
                    .download_limit(&matcher_main, api_version, auth)
                    .map(|d| d as u8),
            )
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
