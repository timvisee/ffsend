// TODO: define redirect policy

use reqwest::{Client, StatusCode};
use reqwest::header::Authorization;

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

/// The default download count.
const PARAMS_DEFAULT_DOWNLOAD: u8 = 1;

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

        // Create a key set for the file
        let key = KeySet::from(self.file, None);

        // Fetch the authentication nonce if not set yet
        if self.nonce.is_empty() {
            self.nonce = self.fetch_auth_nonce(client)?;
        }

        // Compute a signature
        let sig = signature_encoded(key.auth_key().unwrap(), &self.nonce)
            .map_err(|_| PrepareError::ComputeSignature)?;

        // TODO: can we remove this?
        // // Derive a new authentication key
        // key.derive_auth_password(self.password, &self.file.download_url(true));

        // Wrap the parameters data
        let data = OwnedData::from(self.params.clone(), &self.file)
            .map_err(|err| -> PrepareError { err.into() })?;

        // Send the request to change the parameters
        self.change_params(client, data, sig)
            .map_err(|err| err.into())
    }

    /// Fetch the authentication nonce for the file from the Send server.
    fn fetch_auth_nonce(&self, client: &Client)
        -> Result<Vec<u8>, AuthError>
    {
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

    /// Send the request for changing the parameters.
    fn change_params(
        &self,
        client: &Client,
        data: OwnedData<ParamsData>,
        sig: String,
    ) -> Result<(), ChangeError> {
        // Get the params URL, and send the change
        let url = self.file.api_params_url();
        let response = client.post(url)
            .json(&data)
            .header(Authorization(
                format!("send-v1 {}", sig)
            ))
            .send()
            .map_err(|_| ChangeError::Request)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            return Err(ChangeError::RequestStatus(status, status.err_text()).into());
        }

        Ok(())
    }
}

/// The parameters data object, that is sent to the server.
#[derive(Clone, Debug, Serialize)]
pub struct ParamsData {
    /// The number of times this file may be downloaded.
    /// This value must be in the `(0,20)` bounds, as enforced by Send servers.
    #[serde(rename = "dlimit")]
    downloads: Option<u8>,
}

impl ParamsData {
    /// Create a new parameters data object, with the given parameters.
    // TODO: the downloads must be between bounds
    pub fn from(downloads: Option<u8>) -> Self {
        ParamsData {
            downloads,
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
    pub fn set_downloads(&mut self, downloads: Option<u8>)
        -> Result<(), ParamsDataError>
    {
        // Check the download bounds
        if let Some(d) = downloads {
            if d < PARAMS_DOWNLOAD_MIN || d > PARAMS_DOWNLOAD_MAX {
                return Err(ParamsDataError::DownloadBounds);
            }
        }

        // Set the downloads
        self.downloads = downloads;
        Ok(())
    }

    /// Check whether this parameters object is empty,
    /// and wouldn't change any parameter on the server when sent.
    /// Sending an empty parameter data object would thus be useless.
    pub fn is_empty(&self) -> bool {
        self.downloads.is_none()
    }
}

impl Default for ParamsData {
    fn default() -> ParamsData {
        ParamsData {
            downloads: Some(PARAMS_DEFAULT_DOWNLOAD),
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

impl From<AuthError> for Error {
    fn from(err: AuthError) -> Error {
        PrepareError::Auth(err).into()
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
    Auth(#[cause] AuthError),

    /// An error occurred while computing the cryptographic signature.
    #[fail(display = "Failed to compute cryptographic signature")]
    ComputeSignature,

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
pub enum ChangeError {
    /// Sending the request to change the parameters failed.
    #[fail(display = "Failed to send parameter change request")]
    Request,

    /// The response for changing the parameters indicated an error and wasn't
    /// successful.
    #[fail(display = "Bad HTTP response '{}' while changing the parameters", _1)]
    RequestStatus(StatusCode, String),
}
