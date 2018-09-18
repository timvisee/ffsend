use chrono::Duration;
use clap::ArgMatches;
use ffsend_api::config::SEND_DEFAULT_EXPIRE_TIME;
use prettytable::{cell::Cell, format::FormatBuilder, row::Row, Table};

use cmd::matcher::{debug::DebugMatcher, main::MainMatcher, Matcher};
use error::ActionError;
use util::{features_list, format_bool, format_duration};

/// A file debug action.
pub struct Debug<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Debug<'a> {
    /// Construct a new debug action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the debug action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), ActionError> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_debug = DebugMatcher::with(self.cmd_matches).unwrap();

        // Create a table for all debug information
        let mut table = Table::new();
        table.set_format(FormatBuilder::new().padding(0, 2).build());

        // The default host
        table.add_row(Row::new(vec![
            Cell::new("Host:"),
            Cell::new(matcher_debug.host().as_str()),
        ]));

        // The history file
        #[cfg(feature = "history")]
        table.add_row(Row::new(vec![
            Cell::new("History file:"),
            Cell::new(matcher_main.history().to_str().unwrap_or("?")),
        ]));

        // The default host
        table.add_row(Row::new(vec![
            Cell::new("Default expiry:"),
            Cell::new(&format_duration(Duration::seconds(
                SEND_DEFAULT_EXPIRE_TIME,
            ))),
        ]));

        // Render a list of compiled features
        table.add_row(Row::new(vec![
            Cell::new("Features:"),
            Cell::new(&features_list().join(", ")),
        ]));

        // Show whether verbose is used
        table.add_row(Row::new(vec![
            Cell::new("Verbose:"),
            Cell::new(format_bool(matcher_main.verbose())),
        ]));

        // Print the debug table
        table.printstd();

        Ok(())
    }
}
