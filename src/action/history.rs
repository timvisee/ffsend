use clap::ArgMatches;
use failure::Fail;
use prettytable::{format::FormatBuilder, Cell, Row, Table};

use crate::cmd::matcher::{history::HistoryMatcher, main::MainMatcher, Matcher};
use crate::error::ActionError;
use crate::history::{History as HistoryManager, LoadError as HistoryLoadError};
use crate::util::{format_duration, quit_error, quit_error_msg, ErrorHintsBuilder};

/// A history action.
pub struct History<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> History<'a> {
    /// Construct a new history action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the history action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_history = HistoryMatcher::with(self.cmd_matches).unwrap();

        // Get the history path, make sure it exists
        let history_path = matcher_main.history();
        if !history_path.is_file() {
            if !matcher_main.quiet() {
                eprintln!("No files in history");
            }
            return Ok(());
        }

        // History
        let mut history = HistoryManager::load(history_path)?;

        // Do not report any files if there aren't any
        if history.files().is_empty() {
            if !matcher_main.quiet() {
                eprintln!("No files in history");
            }
            return Ok(());
        }

        // Clear all history
        if matcher_history.clear() {
            history.clear();

            // Save history
            if let Err(err) = history.save() {
                quit_error(
                    err,
                    ErrorHintsBuilder::default().verbose(true).build().unwrap(),
                );
            }

            eprintln!("History cleared");
            return Ok(());
        }

        // Remove history item
        if let Some(url) = matcher_history.rm() {
            // Remove item, print error if no item with URL was found
            match history.remove_url(url) {
                Ok(removed) if !removed => quit_error_msg(
                    "could not remove item from history, no item matches given URL",
                    ErrorHintsBuilder::default().verbose(true).build().unwrap(),
                ),
                Err(err) => quit_error(
                    err.context("could not remove item from history"),
                    ErrorHintsBuilder::default().verbose(true).build().unwrap(),
                ),
                _ => {}
            }

            // Save history
            if let Err(err) = history.save() {
                quit_error(
                    err,
                    ErrorHintsBuilder::default().verbose(true).build().unwrap(),
                );
            }

            eprintln!("Item removed from history");
            return Ok(());
        }

        // Get the list of files, and sort the first expiring files to be last
        let mut files = history.files().clone();
        files.sort_by(|a, b| b.expire_at().cmp(&a.expire_at()));

        // Log a history table, or just the URLs in quiet mode
        if !matcher_main.quiet() {
            // Build the list of column names
            let mut columns = vec!["#", "LINK", "EXPIRE"];
            if matcher_main.verbose() {
                columns.push("OWNER TOKEN");
            }

            // Create a new table
            let mut table = Table::new();
            table.set_format(FormatBuilder::new().padding(0, 2).build());
            table.add_row(Row::new(columns.into_iter().map(Cell::new).collect()));

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

                // Define the cell values
                let mut cells: Vec<String> =
                    vec![format!("{}", i + 1), file.download_url(true).into(), expiry];
                if matcher_main.verbose() {
                    cells.push(owner_token);
                }

                // Add the row
                table.add_row(Row::new(cells.into_iter().map(|c| Cell::new(&c)).collect()));
            }

            // Print the table
            table.printstd();
        } else {
            files
                .iter()
                .for_each(|f| println!("{}", f.download_url(true)));
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
