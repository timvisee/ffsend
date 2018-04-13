use failure::Error as FailureError;
use openssl::symm::decrypt_aead;
use reqwest::Client;
use reqwest::header::Authorization;
use serde_json;

use api::nonce::{header_nonce, NonceError, request_nonce};
use api::request::{ensure_success, ResponseError};
use api::url::UrlBuilder;
use crypto::b64;
use crypto::key_set::KeySet;
use crypto::sig::signature_encoded;
use file::metadata::Metadata as MetadataData;
use file::remote_file::RemoteFile;
use super::exists::{
    Error as ExistsError,
    Exists as ExistsAction,
};

/// An action to fetch file metadata.
pub struct Metadata<'a> {
    /// The remote file to fetch the metadata for.
    file: &'a RemoteFile,

    /// An optional password to decrypt a protected file.
    password: Option<String>,

    /// Check whether the file exists (recommended).
    check_exists: bool,
}

impl<'a> Metadata<'a> {
    /// Construct a new metadata action.
    pub fn new(
        file: &'a RemoteFile,
        password: Option<String>,
        check_exists: bool,
    ) -> Self {
        Self {
            file,
            password,
            check_exists,
        }
    }

    /// Invoke the metadata action.
    pub fn invoke(self, client: &Client) -> Result<MetadataResponse, Error> {
        // Make sure the given file exists
        if self.check_exists {
            let exist_response = ExistsAction::new(&self.file)
                .invoke(&client)?;

            // Return an error if the file does not exist
            if !exist_response.exists() {
                return Err(Error::Expired);
            }

            // Make sure a password is given when it is required
            if !self.password.is_some() && exist_response.has_password() {
                return Err(Error::PasswordRequired);
            }
        }

        // Create a key set for the file
        let mut key = KeySet::from(self.file, self.password.as_ref());

        // Fetch the authentication nonce
        let auth_nonce = self.fetch_auth_nonce(client)?;

        // Fetch the metadata and the metadata nonce, return the result
        self.fetch_metadata(&client, &mut key, auth_nonce)
            .map_err(|err| err.into())
    }

    /// Fetch the authentication nonce for the file from the remote server.
    fn fetch_auth_nonce(&self, client: &Client)
        -> Result<Vec<u8>, RequestError>
    {
        request_nonce(
            client,
            UrlBuilder::download(self.file, false),
        ).map_err(|err| RequestError::Auth(err))
    }

    /// Create a metadata nonce, and fetch the metadata for the file from the
    /// Send server.
    ///
    /// The key set, along with the authentication nonce must be given.
    ///
    /// The metadata, with the meta nonce is returned.
    fn fetch_metadata(
        &self,
        client: &Client,
        key: &KeySet,
        auth_nonce: Vec<u8>,
    ) -> Result<MetadataResponse, MetaError> {
        // Compute the cryptographic signature for authentication
        let sig = signature_encoded(key.auth_key().unwrap(), &auth_nonce)
            .map_err(|_| MetaError::ComputeSignature)?;

        // Build the request, fetch the encrypted metadata
        let mut response = client.get(UrlBuilder::api_metadata(self.file))
            .header(Authorization(
                format!("send-v1 {}", sig)
            ))
            .send()
            .map_err(|_| MetaError::NonceRequest)?;

        // Ensure the status code is successful
        ensure_success(&response)
            .map_err(|err| MetaError::NonceResponse(err))?;

        // Get the metadata nonce
        let nonce = header_nonce(&response)
            .map_err(|err| MetaError::Nonce(err))?;

        // Parse the metadata response, and decrypt it
        Ok(MetadataResponse::from(
            response.json::<RawMetadataResponse>()
                .map_err(|_| MetaError::Malformed)?
                .decrypt_metadata(&key)
                .map_err(|_| MetaError::Decrypt)?,
            nonce,
        ))
    }
}

/// The metadata response from the server, when fetching the data through
/// the API.
/// This response contains raw metadata, which is still encrypted.
#[derive(Debug, Deserialize)]
pub struct RawMetadataResponse {
    /// The encrypted metadata.
    #[serde(rename = "metadata")]
    meta: String,
}

impl RawMetadataResponse {
    /// Get and decrypt the metadata, based on the raw data in this response.
    ///
    /// The decrypted data is verified using an included tag.
    /// If verification failed, an error is returned.
    pub fn decrypt_metadata(&self, key_set: &KeySet) -> Result<MetadataData, FailureError> {
        // Decode the metadata
        let raw = b64::decode(&self.meta)?;

        // Get the encrypted metadata, and it's tag
        let (encrypted, tag) = raw.split_at(raw.len() - 16);
        // TODO: is the tag length correct, remove assert if it is
        assert_eq!(tag.len(), 16);

        // Decrypt the metadata
		let meta = decrypt_aead(
			KeySet::cipher(),
			key_set.meta_key().unwrap(),
			Some(key_set.iv()),
			&[],
			encrypted,
			&tag,
		)?;

        // Parse the metadata, and return
        Ok(serde_json::from_slice(&meta)?)
    }
}

/// The decoded and decrypted metadata response, holding all the properties.
/// This response object is returned from this action.
pub struct MetadataResponse {
    /// The actual metadata.
    metadata: MetadataData,

    /// The metadata nonce.
    nonce: Vec<u8>,
}

impl<'a> MetadataResponse {
    /// Construct a new response with the given metadata and nonce.
    pub fn from(metadata: MetadataData, nonce: Vec<u8>) -> Self {
        MetadataResponse {
            metadata,
            nonce,
        }
    }

    /// Get the metadata.
    pub fn metadata(&self) -> &MetadataData {
        &self.metadata
    }

    /// Get the nonce.
    pub fn nonce(&self) -> &Vec<u8> {
        &self.nonce
    }
}

#[derive(Fail, Debug)]
pub enum Error {
    /// An error occurred while checking whether the file exists on the
    /// server.
    #[fail(display = "Failed to check whether the file exists")]
    Exists(#[cause] ExistsError),

    /// A general error occurred while requesting the file data.
    /// This may be because authentication failed, because decrypting the
    /// file metadata didn't succeed, or due to some other reason.
    #[fail(display = "Failed to request file data")]
    Request(#[cause] RequestError),

    /// The given Send file has expired, or did never exist in the first place.
    /// Therefore the file could not be downloaded.
    #[fail(display = "The file has expired or did never exist")]
    Expired,

    /// A password is required, but was not given.
    #[fail(display = "Missing password, password required")]
    PasswordRequired,
}

impl From<ExistsError> for Error {
    fn from(err: ExistsError) -> Error {
        Error::Exists(err)
    }
}

impl From<RequestError> for Error {
    fn from(err: RequestError) -> Error {
        Error::Request(err)
    }
}

impl From<MetaError> for Error {
    fn from(err: MetaError) -> Error {
        Error::Request(RequestError::Meta(err))
    }
}

#[derive(Fail, Debug)]
pub enum RequestError {
    /// Failed authenticating, in order to fetch the file data.
    #[fail(display = "Failed to authenticate")]
    Auth(#[cause] NonceError),

    /// Failed to retrieve the file metadata.
    #[fail(display = "Failed to retrieve file metadata")]
    Meta(#[cause] MetaError),
}

#[derive(Fail, Debug)]
pub enum MetaError {
    /// An error occurred while computing the cryptographic signature used for
    /// decryption.
    #[fail(display = "Failed to compute cryptographic signature")]
    ComputeSignature,

    /// Sending the request to gather the metadata encryption nonce failed.
    #[fail(display = "Failed to request metadata nonce")]
    NonceRequest,

    /// The server responded with an error while fetching the metadata
    /// encryption nonce.
    #[fail(display = "Bad response from server while fetching metadata nonce")]
    NonceResponse(#[cause] ResponseError),

    /// Couldn't parse the metadata encryption nonce.
    #[fail(display = "Failed to parse the metadata encryption nonce")]
    Nonce(#[cause] NonceError),

    /// The received metadata is malformed, and couldn't be decoded or
    /// interpreted.
    #[fail(display = "Received malformed metadata")]
    Malformed,

    /// Failed to decrypt the received metadata.
    #[fail(display = "Failed to decrypt received metadata")]
    Decrypt,
}
