use reqwest::{Client, StatusCode};

use api::url::UrlBuilder;
use ext::status_code::StatusCodeExt;
use file::remote_file::RemoteFile;

/// The HTTP status code that is returned for expired files.
const FILE_EXPIRED_STATUS: StatusCode = StatusCode::NotFound;

/// An action to check whether a remote file exists.
/// This aciton returns an `ExistsResponse`, that defines whether the file
/// exists, and whether it is protected by a password.
pub struct Exists<'a> {
    /// The remote file to check.
    file: &'a RemoteFile,
}

impl<'a> Exists<'a> {
    /// Construct a new exists action.
    pub fn new(file: &'a RemoteFile) -> Self {
        Self {
            file,
        }
    }

    /// Invoke the exists action.
    pub fn invoke(self, client: &Client) -> Result<ExistsResponse, Error> {
        self.check_exists(&client)
    }

    /// Send a request to check whether the file exists
    fn check_exists(&self, client: &Client) -> Result<ExistsResponse, Error> {
        // Get the download url, and parse the nonce
        let exists_url = UrlBuilder::api_exists(self.file);
        let mut response = client.get(exists_url)
            .send()
            .map_err(|_| Error::Request)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            // Handle expired files
            if status == FILE_EXPIRED_STATUS {
                return Ok(ExistsResponse::new(false, false));
            } else {
                return Err(Error::RequestStatus(status, status.err_text()).into());
            }
        }

        // Parse the response
        let mut response = response.json::<ExistsResponse>()
            .map_err(|_| Error::Malformed)?;
        response.set_exists(true);

        // TODO: fetch the metadata nonce from the response headers

        Ok(response)
    }
}

/// The exists response.
#[derive(Debug, Deserialize)]
pub struct ExistsResponse {
    /// Whether the file exists.
    #[serde(skip)]
    exists: bool,

    /// Whether this file requires a password.
    #[serde(rename = "password")]
    has_password: bool,
}

impl ExistsResponse {
    /// Construct a new response.
    pub fn new(exists: bool, has_password: bool) -> Self {
        ExistsResponse {
            exists,
            has_password,
        }
    }

    /// Whether the remote file exists on the server.
    pub fn exists(&self) -> bool {
        self.exists
    }

    /// Set whether the remote file exists.
    pub fn set_exists(&mut self, exists: bool) {
        self.exists = exists;
    }

    /// Whether the remote file is protected by a password.
    pub fn has_password(&self) -> bool {
        self.has_password
    }
}

impl Default for ExistsResponse {
    fn default() -> Self {
        ExistsResponse {
            exists: false,
            has_password: false,
        }
    }
}

#[derive(Fail, Debug)]
pub enum Error {
    /// Sending the request to check whether the file exists failed.
    #[fail(display = "Failed to send request whether the file exists")]
    Request,

    /// The response for checking whether the file exists indicated an error
    /// and wasn't successful.
    #[fail(display = "Bad HTTP response '{}' while requesting whether the file exists", _1)]
    RequestStatus(StatusCode, String),

    /// The response from the server when checking if the file exists was
    /// malformed.
    /// Maybe the server responded with a new format that isn't supported yet
    /// by this client.
    #[fail(display = "Received malformed authentication nonce")]
    Malformed,
}
