use reqwest::Client;

use api::data::{
    Error as DataError,
    OwnedData,
};
use api::nonce::{NonceError, request_nonce};
use api::request::{ensure_success, ResponseError};
use api::url::UrlBuilder;
use file::remote_file::RemoteFile;

/// The default download count.
pub const PARAMS_DEFAULT_DOWNLOAD: u8 = 1;
pub const PARAMS_DEFAULT_DOWNLOAD_STR: &'static str = "1";

/// The minimum allowed number of downloads, enforced by the server.
pub const PARAMS_DOWNLOAD_MIN: u8 = 1;

/// The maximum (inclusive) allowed number of downloads,
/// enforced by the server.
pub const PARAMS_DOWNLOAD_MAX: u8 = 20;

/// An action to set parameters for a shared file.
pub struct Params<'a> {
    /// The remote file to change the parameters for.
    file: &'a RemoteFile,

    /// The parameter data that is sent to the server.
    params: ParamsData,

    /// The authentication nonce.
    /// May be an empty vector if the nonce is unknown.
    nonce: Vec<u8>,
}

impl<'a> Params<'a> {
    /// Construct a new parameters action for the given remote file.
    pub fn new(
        file: &'a RemoteFile,
        params: ParamsData,
        nonce: Option<Vec<u8>>,
    ) -> Self {
        Self {
            file,
            params,
            nonce: nonce.unwrap_or(Vec::new()),
        }
    }

    /// Invoke the parameters action.
    pub fn invoke(mut self, client: &Client) -> Result<(), Error> {
        // TODO: validate that the parameters object isn't empty

        // Fetch the authentication nonce if not set yet
        if self.nonce.is_empty() {
            self.nonce = self.fetch_auth_nonce(client)?;
        }

        // Wrap the parameters data
        let data = OwnedData::from(self.params.clone(), &self.file)
            .map_err(|err| -> PrepareError { err.into() })?;

        // Send the request to change the parameters
        self.change_params(client, data)
            .map_err(|err| err.into())
    }

    /// Fetch the authentication nonce for the file from the remote server.
    fn fetch_auth_nonce(&self, client: &Client)
        -> Result<Vec<u8>, PrepareError>
    {
        request_nonce(
            client,
            UrlBuilder::download(self.file, false),
        ).map_err(|err| PrepareError::Auth(err))
    }

    /// Send the request for changing the parameters.
    fn change_params(
        &self,
        client: &Client,
        data: OwnedData<ParamsData>,
    ) -> Result<(), ChangeError> {
        // Get the params URL, and send the change
        let url = UrlBuilder::api_params(self.file);
        let response = client.post(url)
            .json(&data)
            .send()
            .map_err(|_| ChangeError::Request)?;

        // Ensure the response is successful
        ensure_success(&response)
            .map_err(|err| ChangeError::Response(err))
    }
}

/// The parameters data object, that is sent to the server.
// TODO: make sure downloads are in-bound when using the builder
#[derive(Clone, Debug, Builder, Serialize)]
pub struct ParamsData {
    /// The number of times this file may be downloaded.
    /// This value must be in the `(0,20)` bounds, as enforced by Send servers.
    #[serde(rename = "dlimit")]
    download_limit: Option<u8>,
}

impl ParamsData {
    /// Construct a new parameters object, that is empty.
    pub fn new() -> Self {
        ParamsData {
            download_limit: None,
        }
    }

    /// Create a new parameters data object, with the given parameters.
    // TODO: the downloads must be between bounds
    pub fn from(download_limit: Option<u8>) -> Self {
        ParamsData {
            download_limit,
        }
    }

    /// Set the maximum number of allowed downloads, after which the file
    /// will be removed.
    ///
    /// `None` may be given, to keep this parameter as is.
    ///
    /// An error may be returned if the download value is out of the allowed
    /// bound. These bounds are fixed and enforced by the server.
    /// See `PARAMS_DOWNLOAD_MIN` and `PARAMS_DOWNLOAD_MAX`.
    pub fn set_download_limit(&mut self, download_limit: Option<u8>)
        -> Result<(), ParamsDataError>
    {
        // Check the download limit bounds
        if let Some(d) = download_limit {
            if d < PARAMS_DOWNLOAD_MIN || d > PARAMS_DOWNLOAD_MAX {
                return Err(ParamsDataError::DownloadBounds);
            }
        }

        // Set the download limit
        self.download_limit = download_limit;
        Ok(())
    }

    /// Check whether this parameters object is empty,
    /// and wouldn't change any parameter on the server when sent.
    /// Sending an empty parameter data object would thus be useless.
    pub fn is_empty(&self) -> bool {
        self.download_limit.is_none()
    }
}

impl Default for ParamsData {
    fn default() -> ParamsData {
        ParamsData {
            download_limit: None,
        }
    }
}

#[derive(Fail, Debug)]
pub enum Error {
    /// An error occurred while preparing the action.
    #[fail(display = "Failed to prepare setting the parameters")]
    Prepare(#[cause] PrepareError),

    // /// The given Send file has expired, or did never exist in the first place.
    // /// Therefore the file could not be downloaded.
    // #[fail(display = "The file has expired or did never exist")]
    // Expired,

    /// An error has occurred while sending the parameter change request to
    /// the server.
    #[fail(display = "Failed to send the parameter change request")]
    Change(#[cause] ChangeError),
}

impl From<PrepareError> for Error {
    fn from(err: PrepareError) -> Error {
        Error::Prepare(err)
    }
}

impl From<ChangeError> for Error {
    fn from(err: ChangeError) -> Error {
        Error::Change(err)
    }
}

#[derive(Debug, Fail)]
pub enum ParamsDataError {
    /// The number of downloads is invalid, as it was out of the allowed
    /// bounds. See `PARAMS_DOWNLOAD_MIN` and `PARAMS_DOWNLOAD_MAX`.
    // TODO: use bound values from constants, don't hardcode them here
    #[fail(display = "Invalid number of downloads, must be between 1 and 20")]
    DownloadBounds,

    /// Some error occurred while trying to wrap the parameter data in an
    /// owned object, which is required for authentication on the server.
    /// The wrapped error further described the problem.
    #[fail(display = "")]
    Owned(#[cause] DataError),
}

#[derive(Fail, Debug)]
pub enum PrepareError {
    /// Failed authenticating, needed to change the parameters.
    #[fail(display = "Failed to authenticate")]
    Auth(#[cause] NonceError),

    /// An error occurred while building the parameter data that will be send
    /// to the server.
    #[fail(display = "Invalid parameters")]
    ParamsData(#[cause] ParamsDataError),
}

impl From<DataError> for PrepareError {
    fn from(err: DataError) -> PrepareError {
        PrepareError::ParamsData(ParamsDataError::Owned(err))
    }
}

#[derive(Fail, Debug)]
pub enum ChangeError {
    /// Sending the request to change the parameters failed.
    #[fail(display = "Failed to send parameter change request")]
    Request,

    /// The server responded with an error while changing the file parameters.
    #[fail(display = "Bad response from server while changing parameters")]
    Response(#[cause] ResponseError),
}
