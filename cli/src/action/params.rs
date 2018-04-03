use clap::ArgMatches;
use ffsend_api::action::params::{
    Params as ApiParams,
    ParamsDataBuilder,
};
use ffsend_api::file::remote_file::RemoteFile;
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    params::ParamsMatcher,
};
use error::ActionError;
use util::print_success;

/// A file parameters action.
pub struct Params<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Params<'a> {
    /// Construct a new parameters action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the parameters action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_params = ParamsMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_params.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        // TODO: handle error here
        let file = RemoteFile::parse_url(url, matcher_params.owner())?;

        // TODO: show an informative error if the owner token isn't set

        // Build the parameters data object
        let data = ParamsDataBuilder::default()
            .download_limit(matcher_params.download_limit())
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
