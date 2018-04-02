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

use cmd::cmd_info::CmdInfo;
use util::print_error;

/// A file info action.
pub struct Info<'a> {
    cmd: &'a CmdInfo<'a>,
}

impl<'a> Info<'a> {
    /// Construct a new info action.
    pub fn new(cmd: &'a CmdInfo<'a>) -> Self {
        Self {
            cmd,
        }
    }

    /// Invoke the info action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), Error> {
        // Get the share URL
        let url = self.cmd.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        // TODO: handle error here
        let file = RemoteFile::parse_url(url, self.cmd.owner())?;

        // TODO: show an informative error if the owner token isn't set

        // Make sure the file exists
        let exists_response = ApiExists::new(&file)
            .invoke(&client)?;

        // Make sure the file exists
        if !exists_response.exists() {
            return Err(Error::Expired);
        }

        // TODO: make sure a password is set if required

        // Fetch both file info and metadata
        let info = ApiInfo::new(&file, None).invoke(&client)?;
        // TODO: supply a password here
        let metadata = ApiMetadata::new(&file, None).invoke(&client)
            .map_err(|err| print_error(err.context(
                "Failed to fetch file metadata, showing limited info",
            )))
            .ok();

        // Print the result
        println!("ID: {}", file.id());
        if let Some(metadata) = metadata {
            println!("File name: {}", metadata.metadata().name());
            println!("MIME type: {}", metadata.metadata().mime());
        }
        println!("Downloads: {} of {}", info.download_count(), info.download_limit());
        println!("TTL: {} ms", info.ttl_millis());

        // TODO: show the file size, fetch TTL from metadata?

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "Invalid share URL")]
    InvalidUrl(#[cause] FileParseError),

    /// An error occurred while checking if the file exists.
    #[fail(display = "Failed to check whether the file exists")]
    Exists(#[cause] ExistsError),

    /// An error occurred while fetching the file information.
    #[fail(display = "Failed to fetch file info")]
    Info(#[cause] InfoError),

    /// The given Send file has expired, or did never exist in the first place.
    // TODO: do not return an error, but write to stdout that the file does not exist
    #[fail(display = "The file has expired or did never exist")]
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
