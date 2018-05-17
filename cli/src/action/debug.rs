use chrono::Duration;
use clap::ArgMatches;
use ffsend_api::config::SEND_DEFAULT_EXPIRE_TIME;
use prettytable::{
    cell::Cell,
    format::FormatBuilder,
    row::Row,
    Table,
};

use cmd::matcher::{
    debug::DebugMatcher,
    main::MainMatcher,
    Matcher,
};
use error::ActionError;
use util::format_duration;

/// A file debug action.
pub struct Debug<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Debug<'a> {
    /// Construct a new debug action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
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
            Cell::new("host:"),
            Cell::new(matcher_debug.host().as_str()),
        ]));

        // The history file
        table.add_row(Row::new(vec![
            Cell::new("history file:"),
            Cell::new(matcher_main.history().to_str().unwrap_or("?")),
        ]));

        // The default host
        table.add_row(Row::new(vec![
            Cell::new("default expiry:"),
            Cell::new(&format_duration(Duration::seconds(SEND_DEFAULT_EXPIRE_TIME))),
        ]));

        // Print the debug table
        table.printstd();

        Ok(())
    }
}
