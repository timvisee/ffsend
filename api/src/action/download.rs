// TODO: define redirect policy

use std::fs::File;
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
use crypto::sign::signature_encoded;
use ext::status_code::StatusCodeExt;
use file::file::DownloadFile;
use file::metadata::Metadata;
use reader::{EncryptedFileWriter, ProgressReporter, ProgressWriter};

/// The name of the header that is used for the authentication nonce.
const HEADER_AUTH_NONCE: &'static str = "WWW-Authenticate";

/// The HTTP status code that is returned for expired files.
const FILE_EXPIRED_STATUS: StatusCode = StatusCode::NotFound;

/// A file upload action to a Send server.
pub struct Download<'a> {
    /// The Send file to download.
    file: &'a DownloadFile,
}

impl<'a> Download<'a> {
    /// Construct a new download action for the given file.
    pub fn new(file: &'a DownloadFile) -> Self {
        Self {
            file,
        }
    }

    /// Invoke the download action.
    pub fn invoke(
        self,
        client: &Client,
        reporter: Arc<Mutex<ProgressReporter>>,
    ) -> Result<(), Error> {
        // Create a key set for the file
        let mut key = KeySet::from(self.file);

        // Fetch the authentication nonce
        let auth_nonce = self.fetch_auth_nonce(client)?;

        // Fetch the meta nonce, set the input vector
        let meta_nonce = self.fetch_meta_nonce(&client, &mut key, auth_nonce)?;

        // Open the file we will write to
        // TODO: this should become a temporary file first
        // TODO: use the uploaded file name as default
        let path = "downloaded.zip";
        let out = File::create(path)
            .map_err(|err| Error::File(path.into(), FileError::Create(err)))?;

        // Create the file reader for downloading
        let (reader, len) = self.create_file_reader(&key, meta_nonce, &client)?;

        // Create the file writer
        let writer = self.create_file_writer(
            out,
            len,
            &key,
            reporter.clone(),
        ).map_err(|err| Error::File(path.into(), err))?;

        // Download the file
        self.download(reader, writer, len, reporter)?;

        // TODO: return the file path
        // TODO: return the new remote state (does it still exist remote)

        Ok(())
    }

    /// Fetch the authentication nonce for the file from the Send server.
    fn fetch_auth_nonce(&self, client: &Client)
        -> Result<Vec<u8>, Error>
    {
        // Get the download url, and parse the nonce
        let download_url = self.file.download_url(false);
        let response = client.get(download_url)
            .send()
            .map_err(|_| AuthError::NonceReq)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            // Handle expired files
            if status == FILE_EXPIRED_STATUS {
                return Err(Error::Expired);
            } else {
                return Err(AuthError::NonceReqStatus(status, status.err_text()).into());
            }
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

    /// Fetch the metadata nonce.
    /// This method also sets the input vector on the given key set,
    /// extracted from the metadata.
    ///
    /// The key set, along with the authentication nonce must be given.
    /// The meta nonce is returned.
    fn fetch_meta_nonce(
        &self,
        client: &Client,
        key: &mut KeySet,
        auth_nonce: Vec<u8>,
    ) -> Result<Vec<u8>, MetaError> {
        // Fetch the metadata and the nonce
        let (metadata, meta_nonce) = self.fetch_metadata(client, key, auth_nonce)?;

        // Set the input vector, and return the nonce
        key.set_iv(metadata.iv());
        Ok(meta_nonce)
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
    ) -> Result<(Metadata, Vec<u8>), MetaError> {
        // Compute the cryptographic signature for authentication
        let sig = signature_encoded(key.auth_key().unwrap(), &auth_nonce)
            .map_err(|_| MetaError::ComputeSignature)?;

        // Build the request, fetch the encrypted metadata
        let mut response = client.get(self.file.api_meta_url())
            .header(Authorization(
                format!("send-v1 {}", sig)
            ))
            .send()
            .map_err(|_| MetaError::NonceReq)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            return Err(MetaError::NonceReqStatus(status, status.err_text()));
        }

        // Get the metadata nonce
        let nonce = b64::decode(
            response.headers()
                .get_raw(HEADER_AUTH_NONCE)
                .ok_or(MetaError::NoNonceHeader)?
                .one()
                .ok_or(MetaError::MalformedNonce)
                .and_then(|line| String::from_utf8(line.to_vec())
                    .map_err(|_| MetaError::MalformedNonce)
                )?
                .split_terminator(" ")
                .skip(1)
                .next()
                .ok_or(MetaError::MalformedNonce)?
        ).map_err(|_| MetaError::MalformedNonce)?;

        // Parse the metadata response, and decrypt it
        Ok((
            response.json::<MetadataResponse>()
                .map_err(|_| MetaError::Malformed)?
                .decrypt_metadata(&key)
                .map_err(|_| MetaError::Decrypt)?,
            nonce,
        ))
    }

    /// Make a download request, and create a reader that downloads the
    /// encrypted file.
    ///
    /// The response representing the file reader is returned along with the
    /// length of the reader content.
    fn create_file_reader(
        &self,
        key: &KeySet,
        meta_nonce: Vec<u8>,
        client: &Client,
    ) -> Result<(Response, u64), DownloadError> {
        // Compute the cryptographic signature
        let sig = signature_encoded(key.auth_key().unwrap(), &meta_nonce)
            .map_err(|_| DownloadError::ComputeSignature)?;

        // Build and send the download request
        let response = client.get(self.file.api_download_url())
            .header(Authorization(
                format!("send-v1 {}", sig)
            ))
            .send()
            .map_err(|_| DownloadError::Request)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            return Err(DownloadError::RequestStatus(status, status.err_text()));
        }

        // Get the content length
        // TODO: make sure there is enough disk space
        let len = response.headers().get::<ContentLength>()
            .ok_or(DownloadError::NoLength)?.0;

        Ok((response, len))
    }

    /// Create a file writer.
    ///
    /// This writer will will decrypt the input on the fly, and writes the
    /// decrypted data to the given file.
    fn create_file_writer(
        &self,
        file: File,
        len: u64,
        key: &KeySet,
        reporter: Arc<Mutex<ProgressReporter>>,
    ) -> Result<ProgressWriter<EncryptedFileWriter>, FileError> {
        // Build an encrypted writer
        let mut writer = ProgressWriter::new(
            EncryptedFileWriter::new(
                file,
                len as usize,
                KeySet::cipher(),
                key.file_key().unwrap(),
                key.iv(),
            ).map_err(|_| FileError::EncryptedWriter)?
        ).map_err(|_| FileError::EncryptedWriter)?;

        // Set the reporter
        writer.set_reporter(reporter.clone());

        Ok(writer)
    }

    /// Download the file from the reader, and write it to the writer.
    /// The length of the file must also be given.
    /// The status will be reported to the given progress reporter.
    fn download<R: Read>(
        &self,
        mut reader: R,
        mut writer: ProgressWriter<EncryptedFileWriter>,
        len: u64,
        reporter: Arc<Mutex<ProgressReporter>>,
    ) -> Result<(), DownloadError> {
        // Start the writer
        reporter.lock()
            .map_err(|_| DownloadError::Progress)?
            .start(len);

        // Write to the output file
        io::copy(&mut reader, &mut writer).map_err(|_| DownloadError::Download)?;

        // Finish
        reporter.lock()
            .map_err(|_| DownloadError::Progress)?
            .finish();

        // Verify the writer
        if writer.unwrap().verified() {
            Ok(())
        } else {
            Err(DownloadError::Verify)
        }
    }
}

/// The metadata response from the server, when fetching the data through
/// the API.
/// 
/// This metadata is required to successfully download and decrypt the
/// corresponding file.
#[derive(Debug, Deserialize)]
struct MetadataResponse {
    /// The encrypted metadata.
    #[serde(rename="metadata")]
    meta: String,
}

impl MetadataResponse {
    /// Get and decrypt the metadata, based on the raw data in this response.
    ///
    /// The decrypted data is verified using an included tag.
    /// If verification failed, an error is returned.
    pub fn decrypt_metadata(&self, key_set: &KeySet) -> Result<Metadata, FailureError> {
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

#[derive(Fail, Debug)]
pub enum Error {
    /// A general error occurred while requesting the file data.
    /// This may be because authentication failed, because decrypting the
    /// file metadata didn't succeed, or due to some other reason.
    #[fail(display = "Failed to request file data")]
    Request(#[cause] RequestError),

    /// The given Send file has expired, or did never exist in the first place.
    /// Therefore the file could not be downloaded.
    #[fail(display = "The file has expired or did never exist")]
    Expired,

    /// An error occurred while downloading the file.
    #[fail(display = "Failed to download the file")]
    Download(#[cause] DownloadError),

    /// An error occurred while decrypting the downloaded file.
    #[fail(display = "Failed to decrypt the downloaded file")]
    Decrypt,

    /// An error occurred while opening or writing to the target file.
    // TODO: show what file this is about
    #[fail(display = "Couldn't use the target file at '{}'", _0)]
    File(String, #[cause] FileError),
}

impl From<AuthError> for Error {
    fn from(err: AuthError) -> Error {
        Error::Request(RequestError::Auth(err))
    }
}

impl From<MetaError> for Error {
    fn from(err: MetaError) -> Error {
        Error::Request(RequestError::Meta(err))
    }
}

impl From<DownloadError> for Error {
    fn from(err: DownloadError) -> Error {
        Error::Download(err)
    }
}

#[derive(Fail, Debug)]
pub enum RequestError {
    /// Failed authenticating, in order to fetch the file data.
    #[fail(display = "Failed to authenticate")]
    Auth(#[cause] AuthError),

    /// Failed to retrieve the file metadata.
    #[fail(display = "Failed to retrieve file metadata")]
    Meta(#[cause] MetaError),
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
pub enum MetaError {
    /// An error occurred while computing the cryptographic signature used for
    /// decryption.
    #[fail(display = "Failed to compute cryptographic signature")]
    ComputeSignature,

    /// Sending the request to gather the metadata encryption nonce failed.
    #[fail(display = "Failed to request metadata nonce")]
    NonceReq,

    /// The response for fetching the metadata encryption nonce indicated an
    /// error and wasn't successful.
    #[fail(display = "Bad HTTP response '{}' while requesting metadata nonce", _1)]
    NonceReqStatus(StatusCode, String),

    /// No metadata encryption nonce was included in the response from the
    /// server, it was missing.
    #[fail(display = "Missing metadata nonce in server response")]
    NoNonceHeader,

    /// The metadata encryption nonce from the response malformed or empty.
    /// Maybe the server responded with a new format that isn't supported yet
    /// by this client.
    #[fail(display = "Received malformed metadata nonce")]
    MalformedNonce,

    /// The received metadata is malformed, and couldn't be decoded or
    /// interpreted.
    #[fail(display = "Received malformed metadata")]
    Malformed,

    /// Failed to decrypt the received metadata.
    #[fail(display = "Failed to decrypt received metadata")]
    Decrypt,
}

#[derive(Fail, Debug)]
pub enum DownloadError {
    /// An error occurred while computing the cryptographic signature used for
    /// downloading the file.
    #[fail(display = "Failed to compute cryptographic signature")]
    ComputeSignature,

    /// Sending the request to download the file failed.
    #[fail(display = "Failed to request file download")]
    Request,

    /// The response for downloading the indicated an error and wasn't successful.
    #[fail(display = "Bad HTTP response '{}' while requesting file download", _1)]
    RequestStatus(StatusCode, String),

    /// The length of the file is missing, thus the length of the file to download
    /// couldn't be determined.
    #[fail(display = "Couldn't determine file download length, missing property")]
    NoLength,

    /// Failed to start or update the downloading progress, because of this the
    /// download can't continue.
    #[fail(display = "Failed to update download progress")]
    Progress,

    /// The actual download and decryption process the server.
    /// This covers reading the file from the server, decrypting the file,
    /// and writing it to the file system.
    #[fail(display = "Failed to download the file")]
    Download,

    /// Verifiying the downloaded file failed.
    #[fail(display = "File verification failed")]
    Verify,
}

#[derive(Fail, Debug)]
pub enum FileError {
    /// An error occurred while creating or opening the file to write to.
    #[fail(display = "Failed to create or replace the file")]
    Create(#[cause] IoError),

    /// Failed to create an encrypted writer for the file, which is used to
    /// decrypt the downloaded file.
    #[fail(display = "Failed to create file decryptor")]
    EncryptedWriter,
}
