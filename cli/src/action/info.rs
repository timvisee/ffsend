use chrono::Duration;
use clap::ArgMatches;
use failure::Fail;
use ffsend_api::action::exists::{
    Error as ExistsError,
    Exists as ApiExists,
};
use ffsend_api::action::info::{
    Error as InfoError,
    Info as ApiInfo,
};
use ffsend_api::action::metadata::Metadata as ApiMetadata;
use ffsend_api::file::remote_file::{
    FileParseError,
    RemoteFile,
};
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    info::InfoMatcher,
    main::MainMatcher,
};
use history_tool;
use util::{
    ensure_owner_token,
    ensure_password,
    format_bytes,
    format_duration,
    print_error,
};

/// A file info action.
pub struct Info<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Info<'a> {
    /// Construct a new info action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
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
        let client = Client::new();

        // Parse the remote file based on the share URL, derive the owner token from history
        let mut file = RemoteFile::parse_url(url, matcher_info.owner())?;
        history_tool::derive_owner_token(&matcher_main, &mut file);

        // Ensure the owner token is set
        ensure_owner_token(file.owner_token_mut(), &matcher_main);

        // Check whether the file exists
        let exists = ApiExists::new(&file).invoke(&client)?;
        if !exists.exists() {
            // Remove the file from the history manager if it doesn't exist
            history_tool::remove(&matcher_main, &file);

            return Err(Error::Expired);
        }

        // Get the password
        let mut password = matcher_info.password();

        // Ensure a password is set when required
        ensure_password(&mut password, exists.has_password(), &matcher_main);

        // Fetch both file info and metadata
        let info = ApiInfo::new(&file, None).invoke(&client)?;
        let metadata = ApiMetadata::new(&file, password, false)
            .invoke(&client)
            .map_err(|err| print_error(err.context(
                "failed to fetch file metadata, showing limited info",
            )))
            .ok();

        // Get the TTL duration
        let ttl_millis = info.ttl_millis() as i64;
        let ttl = Duration::milliseconds(ttl_millis);

        // Update file properties
        file.set_expire_duration(ttl);

        // Add the file to the history
        history_tool::add(&matcher_main, file.clone(), true);

        // Print all file details
        println!("ID: {}", file.id());
        if let Some(metadata) = metadata {
            let size = metadata.size();
            println!("Name: {}", metadata.metadata().name());
            if size >= 1024 {
                println!("Size: {} ({} B)", format_bytes(size), size);
            } else {
                println!("Size: {}", format_bytes(size));
            }
            println!("MIME: {}", metadata.metadata().mime());
        }
        println!("Downloads: {} of {}", info.download_count(), info.download_limit());
        if ttl_millis >= 60 * 1000 {
            println!("Expiry: {} ({}s)", format_duration(&ttl), ttl.num_seconds());
        } else {
            println!("Expiry: {}", format_duration(&ttl));
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "invalid share URL")]
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
