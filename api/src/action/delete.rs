// TODO: define redirect policy

use reqwest::{Client, StatusCode};

use api::data::{
    Error as DataError,
    OwnedData,
};
use crypto::b64;
use ext::status_code::StatusCodeExt;
use file::remote_file::RemoteFile;

/// The name of the header that is used for the authentication nonce.
const HEADER_AUTH_NONCE: &'static str = "WWW-Authenticate";

/// An action to delete a remote file.
pub struct Delete<'a> {
    /// The remote file to delete.
    file: &'a RemoteFile,

    /// The authentication nonce.
    /// May be an empty vector if the nonce is unknown.
    nonce: Vec<u8>,
}

impl<'a> Delete<'a> {
    /// Construct a new delete action for the given file.
    pub fn new(file: &'a RemoteFile, nonce: Option<Vec<u8>>) -> Self {
        Self {
            file,
            nonce: nonce.unwrap_or(Vec::new()),
        }
    }

    /// Invoke the delete action.
    pub fn invoke(mut self, client: &Client) -> Result<(), Error> {
        // Fetch the authentication nonce if not set yet
        if self.nonce.is_empty() {
            self.nonce = self.fetch_auth_nonce(client)?;
        }

        // Create owned data, to send to the server for authentication
        let data = OwnedData::from(DeleteData::new(), &self.file)
            .map_err(|err| -> PrepareError { err.into() })?;

        // Send the delete request
        self.request_delete(client, data).map_err(|err| err.into())
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

    /// Send a request to delete the remote file, with the given data.
    fn request_delete(
        &self,
        client: &Client,
        data: OwnedData<DeleteData>,
    ) -> Result<(), DeleteError> {
        // Get the delete URL, and send the request
        let url = self.file.api_delete_url();
        let response = client.post(url)
            .json(&data)
            .send()
            .map_err(|_| DeleteError::Request)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            return Err(DeleteError::RequestStatus(status, status.err_text()).into());
        }

        Ok(())
    }
}

/// The delete data object.
/// This object is currently empty, as no additional data is sent to the
/// server.
#[derive(Debug, Serialize)]
pub struct DeleteData { }

impl DeleteData {
    /// Constructor.
    pub fn new() -> Self {
        DeleteData { }
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

    /// An error has occurred while sending the filedeletion request.
    #[fail(display = "Failed to send the file deletion request")]
    Delete(#[cause] DeleteError),
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

impl From<DeleteError> for Error {
    fn from(err: DeleteError) -> Error {
        Error::Delete(err)
    }
}

#[derive(Debug, Fail)]
pub enum DeleteDataError {
    /// Some error occurred while trying to wrap the deletion data in an
    /// owned object, which is required for authentication on the server.
    /// The wrapped error further described the problem.
    #[fail(display = "")]
    Owned(#[cause] DataError),
}

#[derive(Fail, Debug)]
pub enum PrepareError {
    /// Failed to authenticate
    #[fail(display = "Failed to authenticate")]
    Auth(#[cause] AuthError),

    /// An error occurred while building the deletion data that will be
    /// send to the server.
    #[fail(display = "Invalid parameters")]
    DeleteData(#[cause] DeleteDataError),
}

impl From<DataError> for PrepareError {
    fn from(err: DataError) -> PrepareError {
        PrepareError::DeleteData(DeleteDataError::Owned(err))
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
pub enum DeleteError {
    /// Sending the file deletion request failed.
    #[fail(display = "Failed to send file deletion request")]
    Request,

    /// The response for deleting the file indicated an error and wasn't
    /// successful.
    #[fail(display = "Bad HTTP response '{}' while deleting the file", _1)]
    RequestStatus(StatusCode, String),
}
