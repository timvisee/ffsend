use reqwest::Client;

use api::data::{
    Error as DataError,
    OwnedData,
};
use api::nonce::{NonceError, request_nonce};
use api::request::{ensure_success, ResponseError};
use api::url::UrlBuilder;
use file::remote_file::RemoteFile;

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
            .map_err(|err| PrepareError::DeleteData(
                DeleteDataError::Owned(err),
            ))?;

        // Send the delete request
        self.request_delete(client, data).map_err(|err| err.into())
    }

    /// Fetch the authentication nonce for the file from the remote server.
    fn fetch_auth_nonce(&self, client: &Client)
        -> Result<Vec<u8>, Error>
    {
        request_nonce(
            client,
            UrlBuilder::download(self.file, false),
        ).map_err(|err| err.into())
    }

    /// Send a request to delete the remote file, with the given data.
    fn request_delete(
        &self,
        client: &Client,
        data: OwnedData<DeleteData>,
    ) -> Result<(), Error> {
        // Get the delete URL, and send the request
        let url = UrlBuilder::api_delete(self.file);
        let response = client.post(url)
            .json(&data)
            .send()
            .map_err(|_| DeleteError::Request)?;

        // Ensure the status code is succesful
        ensure_success(&response)
            .map_err(|err| err.into())
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

    /// The given Send file has expired, or did never exist in the first place.
    /// Therefore the file could not be downloaded.
    #[fail(display = "The file has expired or did never exist")]
    Expired,

    /// An error has occurred while sending the filedeletion request.
    #[fail(display = "Failed to send the file deletion request")]
    Delete(#[cause] DeleteError),
}

impl From<NonceError> for Error {
    fn from(err: NonceError) -> Error {
        match err {
            NonceError::Expired => Error::Expired,
            err => Error::Prepare(PrepareError::Auth(err)),
        }
    }
}

impl From<PrepareError> for Error {
    fn from(err: PrepareError) -> Error {
        Error::Prepare(err)
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
    Auth(#[cause] NonceError),

    /// An error occurred while building the deletion data that will be
    /// send to the server.
    #[fail(display = "Invalid parameters")]
    DeleteData(#[cause] DeleteDataError),
}

#[derive(Fail, Debug)]
pub enum DeleteError {
    /// Sending the file deletion request failed.
    #[fail(display = "Failed to send file deletion request")]
    Request,

    /// The server responded with an error while requesting file deletion.
    #[fail(display = "Bad response from server while deleting file")]
    Response(#[cause] ResponseError),
}

impl From<ResponseError> for Error {
    fn from(err: ResponseError) -> Self {
        match err {
            ResponseError::Expired => Error::Expired,
            err => Error::Delete(DeleteError::Response(err)),
        }
    }
}
