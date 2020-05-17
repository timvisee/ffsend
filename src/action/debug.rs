use chrono::Duration;
use clap::ArgMatches;
use ffsend_api::config::SEND_DEFAULT_EXPIRE_TIME;
use prettytable::{format::FormatBuilder, Cell, Row, Table};

use crate::client::to_duration;
use crate::cmd::matcher::{debug::DebugMatcher, main::MainMatcher, Matcher};
use crate::error::ActionError;
#[cfg(feature = "clipboard-bin")]
use crate::util::ClipboardType;
use crate::util::{api_version_list, features_list, format_bool, format_duration};

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

        // The crate version
        table.add_row(Row::new(vec![
            Cell::new("Version:"),
            Cell::new(crate_version!()),
        ]));

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

        // The timeouts
        table.add_row(Row::new(vec![
            Cell::new("Timeout:"),
            Cell::new(
                &to_duration(matcher_main.timeout())
                    .map(|t| {
                        format_duration(
                            Duration::from_std(t).expect("failed to convert timeout duration"),
                        )
                    })
                    .unwrap_or("disabled".into()),
            ),
        ]));
        table.add_row(Row::new(vec![
            Cell::new("Transfer timeout:"),
            Cell::new(
                &to_duration(matcher_main.transfer_timeout())
                    .map(|t| {
                        format_duration(
                            Duration::from_std(t)
                                .expect("failed to convert transfer timeout duration"),
                        )
                    })
                    .unwrap_or("disabled".into()),
            ),
        ]));

        // The default host
        table.add_row(Row::new(vec![
            Cell::new("Default expiry:"),
            Cell::new(&format_duration(Duration::seconds(
                SEND_DEFAULT_EXPIRE_TIME as i64,
            ))),
        ]));

        // Render a list of compiled features
        table.add_row(Row::new(vec![
            Cell::new("Features:"),
            Cell::new(&features_list().join(", ")),
        ]));

        // Render a list of compiled features
        table.add_row(Row::new(vec![
            Cell::new("API support:"),
            Cell::new(&api_version_list().join(", ")),
        ]));

        // Show used crypto backend
        table.add_row(Row::new(vec![
            Cell::new("Crypto backend:"),
            #[cfg(feature = "crypto-ring")]
            Cell::new("ring"),
            #[cfg(feature = "crypto-openssl")]
            Cell::new("OpenSSL"),
        ]));

        // Clipboard information
        #[cfg(feature = "clipboard-bin")]
        table.add_row(Row::new(vec![
            Cell::new("Clipboard:"),
            Cell::new(&format!("{}", ClipboardType::select())),
        ]));

        // Show whether quiet is used
        table.add_row(Row::new(vec![
            Cell::new("Quiet:"),
            Cell::new(format_bool(matcher_main.quiet())),
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
