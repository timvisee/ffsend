use clap::ArgMatches;
use ffsend_api::action::params::{
    Error as ParamsError,
    Params as ApiParams,
    ParamsDataBuilder,
};
use ffsend_api::file::remote_file::RemoteFile;
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    main::MainMatcher,
    params::ParamsMatcher,
};
use error::ActionError;
use history_tool;
use util::{ensure_owner_token, print_success};

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
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_params = ParamsMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_params.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        let mut file = RemoteFile::parse_url(url, matcher_params.owner())?;

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
            history_tool::remove(&matcher_main, &file);
        }
        result?;

        // Update the history
        history_tool::add(&matcher_main, file);

        // Print a success message
        print_success("Parameters set");

        Ok(())
    }
}
