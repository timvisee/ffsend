use clap::ArgMatches;

use cmd::matcher::{
    Matcher,
    history::HistoryMatcher,
    main::MainMatcher,
};
use error::ActionError;
use history::{
    History as HistoryManager,
    LoadError as HistoryLoadError,
};

/// A history action.
pub struct History<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> History<'a> {
    /// Construct a new history action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the history action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let _matcher_history = HistoryMatcher::with(self.cmd_matches).unwrap();

        // Get the history path, make sure it exists
        let history_path = matcher_main.history();
        if !history_path.is_file() {
            return Ok(());
        }

        // History
        let history = HistoryManager::load(history_path)?;

        for file in history.files() {
            println!("- File ID: {}", file.id());
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to load the history.
    #[fail(display = "Failed to load file history")]
    Load(#[cause] HistoryLoadError),
}

impl From<HistoryLoadError> for ActionError {
    fn from(err: HistoryLoadError) -> ActionError {
        ActionError::History(Error::Load(err))
    }
}
