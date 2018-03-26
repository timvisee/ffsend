// TODO: define redirect policy

use std::fs::File;
use std::io::{
    self,
    Error as IoError,
    Read,
};
use std::sync::{Arc, Mutex};

use openssl::symm::decrypt_aead;
use reqwest::{Client, Response, StatusCode};
use reqwest::header::Authorization;
use reqwest::header::ContentLength;
use serde_json;

use crypto::b64;
use crypto::key_set::KeySet;
use crypto::sign::signature_encoded;
use file::file::DownloadFile;
use file::metadata::Metadata;
use reader::{EncryptedFileWriter, ProgressReporter, ProgressWriter};

// TODO: don't use these definitions
pub type Result<T> = ::std::result::Result<T, Error>;
type StdResult<T, E> = ::std::result::Result<T, E>;

/// The name of the header that is used for the authentication nonce.
const HEADER_AUTH_NONCE: &'static str = "WWW-Authenticate";

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
    ) -> Result<()> {
        // Create a key set for the file
        let mut key = KeySet::from(self.file);

        // Fetch the authentication nonce
        let auth_nonce = self.fetch_auth_nonce(client)
            .map_err(|err| Error::Request(RequestErr::Auth(err)))?;

        // Fetch the meta nonce, set the input vector
        let meta_nonce = self.fetch_meta_nonce(&client, &mut key, auth_nonce)
            .map_err(|err| Error::Request(RequestErr::Meta(err)))?;

        // Open the file we will write to
        // TODO: this should become a temporary file first
        // TODO: use the uploaded file name as default
        let out = File::create("downloaded.zip")
            .map_err(|err| Error::File(FileError::Create(err)))?;

        // Create the file reader for downloading
        let (reader, len) = self.create_file_reader(&key, meta_nonce, &client)
            .map_err(|err| Error::Download(err))?;

        // Create the file writer
        let writer = self.create_file_writer(
            out,
            len,
            &key,
            reporter.clone(),
        ).map_err(|err| Error::File(err))?;

        // Download the file
        self.download(reader, writer, len, reporter)
            .map_err(|err| Error::Download(err))?;

        // TODO: return the file path
        // TODO: return the new remote state (does it still exist remote)

        Ok(())
    }

    /// Fetch the authentication nonce for the file from the Send server.
    fn fetch_auth_nonce(&self, client: &Client)
        -> StdResult<Vec<u8>, AuthErr>
    {
        // Get the download url, and parse the nonce
        let download_url = self.file.download_url(false);
        let response = client.get(download_url)
            .send()
            .map_err(|_| AuthErr::NonceReq)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            return Err(AuthErr::NonceReqStatus(status, status.err_text()));
        }

        // Get the authentication nonce
        b64::decode(
            response.headers()
                .get_raw(HEADER_AUTH_NONCE)
                .ok_or(AuthErr::NoNonceHeader)?
                .one()
                .ok_or(AuthErr::MalformedNonce)
                .and_then(|line| String::from_utf8(line.to_vec())
                    .map_err(|_| AuthErr::MalformedNonce)
                )?
                .split_terminator(" ")
                .skip(1)
                .next()
                .ok_or(AuthErr::MalformedNonce)?
        ).map_err(|_| AuthErr::MalformedNonce)
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
    ) -> StdResult<Vec<u8>, MetaErr> {
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
    ) -> StdResult<(Metadata, Vec<u8>), MetaErr> {
        // Compute the cryptographic signature for authentication
        let sig = signature_encoded(key.auth_key().unwrap(), &auth_nonce)
            .map_err(|_| MetaErr::ComputeSignature)?;

        // Build the request, fetch the encrypted metadata
        let mut response = client.get(self.file.api_meta_url())
            .header(Authorization(
                format!("send-v1 {}", sig)
            ))
            .send()
            .map_err(|_| MetaErr::NonceReq)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            return Err(MetaErr::NonceReqStatus(status, status.err_text()));
        }

        // Get the metadata nonce
        let nonce = b64::decode(
            response.headers()
                .get_raw(HEADER_AUTH_NONCE)
                .ok_or(MetaErr::NoNonceHeader)?
                .one()
                .ok_or(MetaErr::MalformedNonce)
                .and_then(|line| String::from_utf8(line.to_vec())
                    .map_err(|_| MetaErr::MalformedNonce)
                )?
                .split_terminator(" ")
                .skip(1)
                .next()
                .ok_or(MetaErr::MalformedNonce)?
        ).map_err(|_| MetaErr::MalformedNonce)?;

        // Parse the metadata response, and decrypt it
        Ok((
            response.json::<MetadataResponse>()
                .map_err(|_| MetaErr::Malformed)?
                .decrypt_metadata(&key)
                .map_err(|_| MetaErr::Decrypt)?,
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
    ) -> StdResult<(Response, u64), DownloadErr> {
        // Compute the cryptographic signature
        let sig = signature_encoded(key.auth_key().unwrap(), &meta_nonce)
            .map_err(|_| DownloadErr::ComputeSignature)?;

        // Build and send the download request
        let response = client.get(self.file.api_download_url())
            .header(Authorization(
                format!("send-v1 {}", sig)
            ))
            .send()
            .map_err(|_| DownloadErr::Request)?;

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            return Err(DownloadErr::RequestStatus(status, status.err_text()));
        }

        // Get the content length
        // TODO: make sure there is enough disk space
        let len = response.headers().get::<ContentLength>()
            .ok_or(DownloadErr::NoLength)?.0;

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
    ) -> StdResult<ProgressWriter<EncryptedFileWriter>, FileError> {
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
    ) -> StdResult<(), DownloadErr> {
        // Start the writer
        reporter.lock()
            .map_err(|_| DownloadErr::Progress)?
            .start(len);

        // Write to the output file
        io::copy(&mut reader, &mut writer).map_err(|_| DownloadErr::Download)?;

        // Finish
        reporter.lock()
            .map_err(|_| DownloadErr::Progress)?
            .finish();

        // Verify the writer
        // TODO: delete the file if verification failed, show a proper error
        if writer.unwrap().verified() {
            Ok(())
        } else {
            Err(DownloadErr::Verify)
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
    // TODO: do not unwrap, return a proper error
    pub fn decrypt_metadata(&self, key_set: &KeySet) -> Result<Metadata> {
        // Decode the metadata
        let raw = b64::decode(&self.meta)
            .expect("failed to decode metadata from server");

        // Get the encrypted metadata, and it's tag
        let (encrypted, tag) = raw.split_at(raw.len() - 16);
        // TODO: is the tag length correct, remove assert if it is
        assert_eq!(tag.len(), 16);

        // Decrypt the metadata
        // TODO: do not unwrap, return an error
		let meta = decrypt_aead(
			KeySet::cipher(),
			key_set.meta_key().unwrap(),
			Some(key_set.iv()),
			&[],
			encrypted,
			&tag,
		).expect("failed to decrypt metadata, invalid tag?");

        // Parse the metadata, and return
        Ok(
            serde_json::from_slice(&meta)
                .expect("failed to parse decrypted metadata as JSON")
        )
    }
}

#[derive(Fail, Debug)]
pub enum Error {
    /// A general error occurred while requesting the file data.
    /// This may be because authentication failed, because decrypting the
    /// file metadata didn't succeed, or due to some other reason.
    #[fail(display = "Failed to request file data")]
    Request(#[cause] RequestErr),

    /// The given Send file has expired, or did never exist in the first place.
    /// Therefore the file could not be downloaded.
    // TODO: return this error when the file is expired
    #[fail(display = "The file has expired or did never exist")]
    Expired,

    /// An error occurred while downloading the file.
    #[fail(display = "Failed to download the file")]
    Download(#[cause] DownloadErr),

    /// An error occurred while decrypting the downloaded file.
    #[fail(display = "Failed to decrypt the downloaded file")]
    Decrypt,

    // TODO: add description
    // TODO: show what file this is about
    #[fail(display = "Could not open the file for writing")]
    File(#[cause] FileError),
}

#[derive(Fail, Debug)]
pub enum RequestErr {
    /// Failed authenticating, in order to fetch the file data.
    #[fail(display = "Failed to authenticate")]
    Auth(#[cause] AuthErr),

    /// Failed to retrieve the file metadata.
    #[fail(display = "Failed to retrieve file metadata")]
    Meta(#[cause] MetaErr),
}

#[derive(Fail, Debug)]
pub enum AuthErr {
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
pub enum MetaErr {
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
pub enum DownloadErr {
    /// An error occurred while computing the cryptographic signature used for
    /// downloading the file.
    #[fail(display = "Failed to compute cryptographic signature")]
    ComputeSignature,

    /// Sending the request to gather the metadata encryption nonce failed.
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
    #[fail(display = "Failed to create or open file")]
    Create(#[cause] IoError),

    /// Failed to create an encrypted writer for the file, which is used to
    /// decrypt the downloaded file.
    #[fail(display = "Failed to create file decryptor")]
    EncryptedWriter,
}

/// Reqwest status code extention.
trait StatusCodeExt {
    /// Build a basic error message based on the status code.
    fn err_text(&self) -> String;
}

impl StatusCodeExt for StatusCode {
    fn err_text(&self) -> String {
        self.canonical_reason()
            .map(|text| text.to_owned())
            .unwrap_or(format!("{}", self.as_u16()))
    }
}
