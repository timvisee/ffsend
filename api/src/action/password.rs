// TODO: define redirect policy

use std::io::{
    self,
    Error as IoError,
    Read,
};
use std::sync::{Arc, Mutex};

use failure::Error as FailureError;
use openssl::symm::decrypt_aead;
use reqwest::{Client, Response, StatusCode};
use reqwest::header::Authorization;
use reqwest::header::ContentLength;
use serde_json;

use crypto::b64;
use crypto::key_set::KeySet;
use crypto::sig::signature_encoded;
use ext::status_code::StatusCodeExt;
use file::file::DownloadFile;
use file::metadata::Metadata;
use reader::{EncryptedFileWriter, ProgressReporter, ProgressWriter};

/// An action to change a password of an uploaded Send file.
pub struct Password<'a> {
    /// The uploaded file to change the password for.
    file: &'a DownloadFile,

    /// The new password.
    password: &'a str,
}

impl<'a> Password<'a> {
    /// Construct a new password action for the given file.
    pub fn new(file: &'a DownloadFile, password: &'a str) -> Self {
        Self {
            file,
            password,
        }
    }

    /// Invoke the password action.
    // TODO: allow passing an optional existing authentication nonce
    pub fn invoke(self, client: &Client) -> Result<(), Error> {
        // Create a key set for the file
        let mut key = KeySet::from(self.file);

        // Fetch the authentication nonce
        let auth_nonce = self.fetch_auth_nonce(client)?;

        // Compute a signature
        let sig = signature_encoded(key.auth_key().unwrap(), &auth_nonce)
            .map_err(|_| PrepareError::ComputeSignature)?;

        // Derive a new authentication key
        key.derive_auth_password(self.password, self.file.download_url(true));

        // Send the request to change the password
        change_password(client, &key, sig)
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
        key: &KeySet,
        sig: String,
    ) -> Result<Vec<u8>, ChangeError> {
        // Get the password URL, and send the change
        let url = self.file.api_password_url();
        let response = client.post(url)
            .json(PasswordData::from(&key))
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
#[derive(Debug, Serializable)]
struct PasswordData {
    /// The authentication key
    auth: String,
}

impl PasswordData {
    /// Create the password data object from the given key set.
    pub fn from(key: &KeySet) -> PasswordData {
        PasswordData {
            // TODO: do not unwrap here
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

#[derive(Fail, Debug)]
pub enum PrepareError {
    /// Failed authenticating, needed to set a new password.
    #[fail(display = "Failed to authenticate")]
    Auth(#[cause] AuthError),

    /// An error occurred while computing the cryptographic signature.
    #[fail(display = "Failed to compute cryptographic signature")]
    ComputeSignature,
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
