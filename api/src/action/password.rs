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

/// An action to change a password of an uploaded Send file.
pub struct Password<'a> {
    /// The remote file to change the password for.
    file: &'a RemoteFile,

    /// The new password to use for the file.
    password: &'a str,

    /// The authentication nonce.
    /// May be an empty vector if the nonce is unknown.
    nonce: Vec<u8>,
}

impl<'a> Password<'a> {
    /// Construct a new password action for the given remote file.
    pub fn new(
        file: &'a RemoteFile,
        password: &'a str,
        nonce: Option<Vec<u8>>,
    ) -> Self {
        Self {
            file,
            password,
            nonce: nonce.unwrap_or(Vec::new()),
        }
    }

    /// Invoke the password action.
    pub fn invoke(mut self, client: &Client) -> Result<(), Error> {
        // Create a key set for the file
        let mut key = KeySet::from(self.file, None);

        // Fetch the authentication nonce if not set yet
        if self.nonce.is_empty() {
            self.nonce = self.fetch_auth_nonce(client)?;
        }

        // Compute a signature
        let sig = signature_encoded(key.auth_key().unwrap(), &self.nonce)
            .map_err(|_| PrepareError::ComputeSignature)?;

        // Derive a new authentication key
        key.derive_auth_password(self.password, &self.file.download_url(true));

        // Build the password data, wrap it as owned
        let data = OwnedData::from(PasswordData::from(&key), &self.file)
            .map_err(|err| -> PrepareError { err.into() })?;

        // Send the request to change the password
        self.change_password(client, data, sig)
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

    /// Send the request for changing the file password.
    fn change_password(
        &self,
        client: &Client,
        data: OwnedData<PasswordData>,
        sig: String,
    ) -> Result<(), ChangeError> {
        // Get the password URL, and send the change
        let url = self.file.api_password_url();
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

/// The data object to send to the password endpoint,
/// which sets the file password.
#[derive(Debug, Serialize, Deserialize)]
struct PasswordData {
    /// The authentication key
    auth: String,
}

impl PasswordData {
    /// Create the password data object from the given key set.
    pub fn from(key: &KeySet) -> PasswordData {
        PasswordData {
            auth: key.auth_key_encoded().unwrap(),
        }
    }
}

#[derive(Fail, Debug)]
pub enum Error {
    /// An error occurred while preparing the action.
    #[fail(display = "Failed to prepare setting the password")]
    Prepare(#[cause] PrepareError),

    // /// The given Send file has expired, or did never exist in the first place.
    // /// Therefore the file could not be downloaded.
    // #[fail(display = "The file has expired or did never exist")]
    // Expired,

    /// An error has occurred while sending the password change request to
    /// the server.
    #[fail(display = "Failed to send the password change request")]
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

#[derive(Fail, Debug)]
pub enum PrepareError {
    /// Failed authenticating, needed to set a new password.
    #[fail(display = "Failed to authenticate")]
    Auth(#[cause] AuthError),

    /// An error occurred while computing the cryptographic signature.
    #[fail(display = "Failed to compute cryptographic signature")]
    ComputeSignature,

    /// Some error occurred while building the data that will be sent.
    /// The owner token might possibly be missing, the wrapped error will
    /// describe this further.
    #[fail(display = "")]
    Data(#[cause] DataError),
}

impl From<DataError> for PrepareError {
    fn from(err: DataError) -> PrepareError {
        PrepareError::Data(err)
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
    /// Sending the request to change the password failed.
    #[fail(display = "Failed to send password change request")]
    Request,

    /// The response for changing the password indicated an error and wasn't successful.
    #[fail(display = "Bad HTTP response '{}' while changing the password", _1)]
    RequestStatus(StatusCode, String),
}
