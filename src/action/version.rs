use clap::ArgMatches;
use ffsend_api::action::version::{Error as VersionError, Version as ApiVersion};

use crate::client::create_config;
use crate::cmd::matcher::main::MainMatcher;
use crate::cmd::matcher::{version::VersionMatcher, Matcher};
use crate::error::ActionError;

/// A file version action.
pub struct Version<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Version<'a> {
    /// Construct a new version action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the version action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_version = VersionMatcher::with(self.cmd_matches).unwrap();
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();

        // Get the host
        let host = matcher_version.host();

        // Create a reqwest client
        let client_config = create_config(&matcher_main);
        let client = client_config.client(false);

        // Make sure the file version
        let response = ApiVersion::new(host).invoke(&client);

        // Print the result
        match response {
            Ok(v) => println!("API version: {}", v),
            Err(VersionError::Unknown) => println!("Version: unknown"),
            Err(VersionError::Unsupported(v)) => println!("Version: {} (unsupported)", v),
            Err(e) => return Err(e.into()),
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// An error occurred while attempting to determine the Send server version.
    #[fail(display = "failed to check the server version")]
    Version(#[cause] VersionError),
}

impl From<VersionError> for Error {
    fn from(err: VersionError) -> Error {
        Error::Version(err)
    }
}
