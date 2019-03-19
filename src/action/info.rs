use chrono::Duration;
use clap::ArgMatches;
use failure::Fail;
use ffsend_api::action::exists::{Error as ExistsError, Exists as ApiExists};
use ffsend_api::action::info::{Error as InfoError, Info as ApiInfo};
use ffsend_api::action::metadata::Metadata as ApiMetadata;
use ffsend_api::file::remote_file::{FileParseError, RemoteFile};
use prettytable::{format::FormatBuilder, Cell, Row, Table};

use crate::client::create_config;
use crate::cmd::matcher::{info::InfoMatcher, main::MainMatcher, Matcher};
#[cfg(feature = "history")]
use crate::history_tool;
use crate::util::{
    ensure_owner_token, ensure_password, format_bytes, format_duration, print_error,
};

/// A file info action.
pub struct Info<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Info<'a> {
    /// Construct a new info action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the info action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), Error> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_info = InfoMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_info.url();

        // Create a reqwest client
        let client_config = create_config(&matcher_main);
        let client = client_config.client(false);

        // Parse the remote file based on the share URL, derive the owner token from history
        let mut file = RemoteFile::parse_url(url, matcher_info.owner())?;
        #[cfg(feature = "history")]
        history_tool::derive_file_properties(&matcher_main, &mut file);

        // Ask the user to set the owner token for more detailed information
        let has_owner = ensure_owner_token(file.owner_token_mut(), &matcher_main, true);

        // Check whether the file exists
        let exists = ApiExists::new(&file).invoke(&client)?;
        if !exists.exists() {
            // Remove the file from the history manager if it doesn't exist
            #[cfg(feature = "history")]
            history_tool::remove(&matcher_main, &file);

            return Err(Error::Expired);
        }

        // Get the password, ensure the password is set when required
        let mut password = matcher_info.password();
        let has_password = ensure_password(
            &mut password,
            exists.requires_password(),
            &matcher_main,
            true,
        );

        // Fetch both file info and metadata
        let info = if has_owner {
            Some(ApiInfo::new(&file, None).invoke(&client)?)
        } else {
            None
        };
        let metadata = if has_password {
            ApiMetadata::new(&file, password, false)
                .invoke(&client)
                .map_err(|err| {
                    print_error(err.context("failed to fetch file metadata, showing limited info"))
                })
                .ok()
        } else {
            None
        };

        // Update history file TTL if info is known
        if let Some(info) = &info {
            let ttl_millis = info.ttl_millis() as i64;
            let ttl = Duration::milliseconds(ttl_millis);
            file.set_expire_duration(ttl);
        }

        // Add the file to the history
        #[cfg(feature = "history")]
        history_tool::add(&matcher_main, file.clone(), true);

        // Create a new table for the information
        let mut table = Table::new();
        table.set_format(FormatBuilder::new().padding(0, 2).build());

        // Add the ID
        table.add_row(Row::new(vec![Cell::new("ID:"), Cell::new(file.id())]));

        // Show file metadata if available
        if let Some(metadata) = &metadata {
            // The file name
            table.add_row(Row::new(vec![
                Cell::new("Name:"),
                Cell::new(metadata.metadata().name()),
            ]));

            // The file size
            let size = metadata.size();
            table.add_row(Row::new(vec![
                Cell::new("Size:"),
                Cell::new(&if size >= 1024 {
                    format!("{} ({} B)", format_bytes(size), size)
                } else {
                    format_bytes(size)
                }),
            ]));

            // The file MIME
            table.add_row(Row::new(vec![
                Cell::new("MIME:"),
                Cell::new(metadata.metadata().mime()),
            ]));
        }

        // Show file info if available
        if let Some(info) = &info {
            // The download count
            table.add_row(Row::new(vec![
                Cell::new("Downloads:"),
                Cell::new(&format!(
                    "{} of {}",
                    info.download_count(),
                    info.download_limit()
                )),
            ]));

            // The time to live
            let ttl_millis = info.ttl_millis() as i64;
            let ttl = Duration::milliseconds(ttl_millis);
            table.add_row(Row::new(vec![
                Cell::new("Expiry:"),
                Cell::new(&if ttl_millis >= 60 * 1000 {
                    format!("{} ({}s)", format_duration(&ttl), ttl.num_seconds())
                } else {
                    format_duration(&ttl)
                }),
            ]));
        }

        // Print the info table
        table.printstd();

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "invalid share link")]
    InvalidUrl(#[cause] FileParseError),

    /// An error occurred while checking if the file exists.
    #[fail(display = "failed to check whether the file exists")]
    Exists(#[cause] ExistsError),

    /// An error occurred while fetching the file information.
    #[fail(display = "failed to fetch file info")]
    Info(#[cause] InfoError),

    /// The given Send file has expired, or did never exist in the first place.
    #[fail(display = "the file has expired or did never exist")]
    Expired,
}

impl From<FileParseError> for Error {
    fn from(err: FileParseError) -> Error {
        Error::InvalidUrl(err)
    }
}

impl From<ExistsError> for Error {
    fn from(err: ExistsError) -> Error {
        Error::Exists(err)
    }
}

impl From<InfoError> for Error {
    fn from(err: InfoError) -> Error {
        Error::Info(err)
    }
}
