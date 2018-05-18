use clap::ArgMatches;
use prettytable::{
    cell::Cell,
    format::FormatBuilder,
    row::Row,
    Table,
};

use cmd::matcher::{
    Matcher,
    main::MainMatcher,
};
use error::ActionError;
use history::{
    History as HistoryManager,
    LoadError as HistoryLoadError,
};
use util::format_duration;

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

        // Get the history path, make sure it exists
        let history_path = matcher_main.history();
        if !history_path.is_file() {
            eprintln!("No files in history");
            return Ok(());
        }

        // History
        let history = HistoryManager::load(history_path)?;

        // Do not report any files if there aren't any
        if history.files().is_empty() {
            eprintln!("No files in history");
            return Ok(());
        }

        // Create a new table
        let mut table = Table::new();
        table.set_format(FormatBuilder::new().padding(0, 2).build());
        table.add_row(Row::new(vec![
            Cell::new("#"),
            Cell::new("LINK"),
            Cell::new("EXPIRY"),
            Cell::new("OWNER TOKEN"),
        ]));

        // Get the list of files, and sort the first expiring files to be last
        let mut files = history.files().clone();
        files.sort_by(|a, b| b.expire_at().cmp(&a.expire_at()));

        // Add an entry for each file
        for (i, file) in files.iter().enumerate() {
            // Build the expiry time string
            let mut expiry = format_duration(&file.expire_duration());
            if file.expire_uncertain() {
                expiry.insert(0, '~');
            }

            // Get the owner token
            let owner_token: String = match file.owner_token() {
                Some(token) => token.clone(),
                None => "?".into(),
            };

            // Add the row
            table.add_row(Row::new(vec![
                Cell::new(&format!("{}", i + 1)),
                Cell::new(file.download_url(true).as_str()),
                Cell::new(&expiry),
                Cell::new(&owner_token),
            ]));
        }

        // Print the table
        table.printstd();

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
