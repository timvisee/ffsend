use std::fs;
use std::io;

use clap::ArgMatches;

use crate::cmd::matcher::{generate::completions::CompletionsMatcher, main::MainMatcher, Matcher};
use crate::error::ActionError;

/// A file completions action.
pub struct Completions<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Completions<'a> {
    /// Construct a new completions action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the completions action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_completions = CompletionsMatcher::with(self.cmd_matches).unwrap();

        // Obtain shells to generate completions for, build application definition
        let shells = matcher_completions.shells();
        let dir = matcher_completions.output();
        let quiet = matcher_main.quiet();
        let mut app = crate::cmd::handler::Handler::build();

        // If the directory does not exist yet, attempt to create it
        if !dir.is_dir() {
            fs::create_dir_all(&dir).map_err(Error::CreateOutputDir)?;
        }

        // Generate completions
        for shell in shells {
            if !quiet {
                eprint!(
                    "Generating completions for {}...",
                    format!("{}", shell).to_lowercase()
                );
            }
            app.gen_completions(crate_name!(), shell, &dir);
            if !quiet {
                eprintln!(" done.");
            }
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// An error occurred while creating the output directory.
    #[fail(display = "failed to create output directory, it doesn't exist")]
    CreateOutputDir(#[cause] io::Error),
}
