// TODO: define redirect policy

use std::cmp::max;

use reqwest::{
    Client,
    Error as ReqwestError,
    StatusCode,
};
use reqwest::header::Authorization;
use serde_json;

use api::data::{
    Error as DataError,
    OwnedData,
};
use crypto::b64;
use crypto::key_set::KeySet;
use crypto::sig::signature_encoded;
use ext::status_code::StatusCodeExt;
use file::remote_file::RemoteFile;

/// The name of the header that is used for the authentication nonce.
const HEADER_AUTH_NONCE: &'static str = "WWW-Authenticate";

/// An action to fetch info of a shared file.
pub struct Info<'a> {
    /// The remote file to fetch the info for.
    file: &'a RemoteFile,

    // TODO: use this?
    /// The authentication nonce.
    /// May be an empty vector if the nonce is unknown.
    nonce: Vec<u8>,
}

impl<'a> Info<'a> {
    /// Construct a new info action for the given remote file.
    pub fn new(file: &'a RemoteFile, nonce: Option<Vec<u8>>) -> Self {
        Self {
            file,
            nonce: nonce.unwrap_or(Vec::new()),
        }
    }

    /// Invoke the info action.
    pub fn invoke(mut self, client: &Client) -> Result<InfoResponse, Error> {
        // Create a key set for the file
        let key = KeySet::from(self.file, None);

        // Fetch the authentication nonce if not set yet
        if self.nonce.is_empty() {
            self.nonce = self.fetch_auth_nonce(client)?;
        }

        // Compute a signature
        let sig = signature_encoded(key.auth_key().unwrap(), &self.nonce)
            .map_err(|_| PrepareError::ComputeSignature)?;

        // Create owned data, to send to the server for authentication
        let data = OwnedData::from(InfoData::new(), &self.file)
            .map_err(|err| -> PrepareError { err.into() })?;

        // Send the info request
        self.fetch_info(client, data, sig)
            .map_err(|err| err.into())
    }

    /// Fetch the authentication nonce for the file from the remote server.
    fn fetch_auth_nonce(&self, client: &Client) -> Result<Vec<u8>, AuthError> {
        // Get the download URL, and parse the nonce
        let download_url = self.file.download_url(false);
        let response = client.get(download_url)
            .send()
            .map_err(|_| AuthError::NonceReq)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            // TODO: should we check here whether a 404 is returned?
            // // Handle expired files
            // if status == FILE_EXPIRED_STATUS {
            //     return Err(Error::Expired);
            // } else {
            return Err(AuthError::NonceReqStatus(status, status.err_text()).into());
            // }
        }

        // Get the authentication nonce
        b64::decode(
            response.headers()
                .get_raw(HEADER_AUTH_NONCE)
                .ok_or(AuthError::NoNonceHeader)?
                .one()
                .ok_or(AuthError::MalformedNonce)
                .and_then(|line| String::from_utf8(line.to_vec())
                    .map_err(|_| AuthError::MalformedNonce)
                )?
                .split_terminator(" ")
                .skip(1)
                .next()
                .ok_or(AuthError::MalformedNonce)?
        ).map_err(|_| AuthError::MalformedNonce.into())
    }

    /// Send the request for fetching the remote file info.
    fn fetch_info(
        &self,
        client: &Client,
        data: OwnedData<InfoData>,
        sig: String,
    ) -> Result<InfoResponse, InfoError> {
        // Get the info URL, and send the request
        let url = self.file.api_info_url();
        let mut response = client.post(url)
            .json(&data)
            .header(Authorization(
                format!("send-v1 {}", sig)
            ))
            .send()
            .map_err(|_| InfoError::Request)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            return Err(InfoError::RequestStatus(status, status.err_text()).into());
        }

        // Decode the JSON response
        let response: InfoResponse = match response.json() {
            Ok(response) => response,
            Err(err) => return Err(InfoError::Decode(err)),
        };

        Ok(response)
    }
}

/// The info data object.
/// This object is currently empty, as no additional data is sent to the
/// server.
#[derive(Debug, Serialize)]
pub struct InfoData { }

impl InfoData {
    /// Constructor.
    pub fn new() -> Self {
        InfoData { }
    }
}

/// The file info response.
#[derive(Debug, Deserialize)]
pub struct InfoResponse {
    /// The download limit.
    #[serde(rename = "dlimit")]
    download_limit: usize,

    /// The total number of times the file has been downloaded.
    #[serde(rename = "dtotal")]
    download_count: usize,

    /// The time to live for this file in milliseconds.
    #[serde(rename = "ttl")]
    ttl: u64,
}

impl InfoResponse {
    /// Get the number of times this file has been downloaded.
    pub fn download_count(&self) -> usize {
        self.download_count
    }

    /// Get the maximum number of times the file may be downloaded.
    pub fn download_limit(&self) -> usize {
        self.download_limit
    }

    /// Get the number of times this file may still be downloaded.
    pub fn download_left(&self) -> usize {
        max(self.download_limit() - self.download_count(), 0)
    }

    /// Get the time to live for this file, in milliseconds from the time the
    /// request was made.
    pub fn ttl_millis(&self) -> u64 {
        self.ttl
    }
}

#[derive(Fail, Debug)]
pub enum Error {
    /// An error occurred while preparing the action.
    #[fail(display = "Failed to prepare the action")]
    Prepare(#[cause] PrepareError),

    // /// The given Send file has expired, or did never exist in the first place.
    // /// Therefore the file could not be downloaded.
    // #[fail(display = "The file has expired or did never exist")]
    // Expired,

    /// An error has occurred while sending the info request to the server.
    #[fail(display = "Failed to send the file info request")]
    Info(#[cause] InfoError),
}

impl From<PrepareError> for Error {
    fn from(err: PrepareError) -> Error {
        Error::Prepare(err)
    }
}

impl From<AuthError> for Error {
    fn from(err: AuthError) -> Error {
        PrepareError::Auth(err).into()
    }
}

impl From<InfoError> for Error {
    fn from(err: InfoError) -> Error {
        Error::Info(err)
    }
}

#[derive(Debug, Fail)]
pub enum InfoDataError {
    /// Some error occurred while trying to wrap the info data in an
    /// owned object, which is required for authentication on the server.
    /// The wrapped error further described the problem.
    #[fail(display = "")]
    Owned(#[cause] DataError),
}

#[derive(Fail, Debug)]
pub enum PrepareError {
    /// Failed authenticating, needed to fetch the info
    #[fail(display = "Failed to authenticate")]
    Auth(#[cause] AuthError),

    /// An error occurred while computing the cryptographic signature.
    #[fail(display = "Failed to compute cryptographic signature")]
    ComputeSignature,

    /// An error occurred while building the info data that will be
    /// send to the server.
    #[fail(display = "Invalid parameters")]
    InfoData(#[cause] InfoDataError),
}

impl From<DataError> for PrepareError {
    fn from(err: DataError) -> PrepareError {
        PrepareError::InfoData(InfoDataError::Owned(err))
    }
}

#[derive(Fail, Debug)]
pub enum AuthError {
    /// Sending the request to gather the authentication encryption nonce
    /// failed.
    #[fail(display = "Failed to request authentication nonce")]
    NonceReq,

    /// The response for fetching the authentication encryption nonce
    /// indicated an error and wasn't successful.
    #[fail(display = "Bad HTTP response '{}' while requesting authentication nonce", _1)]
    NonceReqStatus(StatusCode, String),

    /// No authentication encryption nonce was included in the response
    /// from the server, it was missing.
    #[fail(display = "Missing authentication nonce in server response")]
    NoNonceHeader,

    /// The authentication encryption nonce from the response malformed or
    /// empty.
    /// Maybe the server responded with a new format that isn't supported yet
    /// by this client.
    #[fail(display = "Received malformed authentication nonce")]
    MalformedNonce,
}

#[derive(Fail, Debug)]
pub enum InfoError {
    /// Sending the request to fetch the file info failed.
    #[fail(display = "Failed to send file info request")]
    Request,

    /// The response fetching the file info indicated an error and wasn't
    /// successful.
    #[fail(display = "Bad HTTP response '{}' while fetching the file info", _1)]
    RequestStatus(StatusCode, String),

    /// Failed to decode the info response from the server.
    /// Maybe the server responded with data from a newer API version.
    #[fail(display = "Failed to decode info response")]
    Decode(#[cause] ReqwestError),
}
