use std::fs::File;
use std::io::{
    BufReader,
    Error as IoError,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use mime_guess::{guess_mime_type, Mime};
use openssl::symm::encrypt_aead;
use reqwest::{
    Client, 
    Error as ReqwestError,
    Request,
    StatusCode,
};
use reqwest::header::Authorization;
use reqwest::mime::APPLICATION_OCTET_STREAM;
use reqwest::multipart::{Form, Part};
use url::{
    ParseError as UrlParseError,
    Url,
};

use crypto::b64;
use crypto::key_set::KeySet;
use ext::status_code::StatusCodeExt;
use file::remote_file::RemoteFile;
use file::metadata::{Metadata, XFileMetadata};
use reader::{
    EncryptedFileReader,
    ExactLengthReader,
    ProgressReader,
    ProgressReporter,
};
use super::params::{
    Error as ParamsError,
    Params,
    ParamsData,
};
use super::password::{
    Error as PasswordError,
    Password,
};

type EncryptedReader = ProgressReader<BufReader<EncryptedFileReader>>;

/// The name of the header that is used for the authentication nonce.
const HEADER_AUTH_NONCE: &'static str = "WWW-Authenticate";

/// A file upload action to a Send server.
pub struct Upload {
    /// The Send host to upload the file to.
    host: Url,

    /// The file to upload.
    path: PathBuf,

    /// The name of the file being uploaded.
    /// This has no relation to the file path, and will become the name of the
    /// shared file if set.
    name: Option<String>,

    /// An optional password to protect the file with.
    password: Option<String>,

    /// Optional file parameters to set.
    params: Option<ParamsData>,
}

impl Upload {
    /// Construct a new upload action.
    pub fn new(
        host: Url,
        path: PathBuf,
        name: Option<String>,
        password: Option<String>,
        params: Option<ParamsData>,
    ) -> Self {
        Self {
            host,
            path,
            name,
            password,
            params,
        }
    }

    /// Invoke the upload action.
    pub fn invoke(
        self,
        client: &Client,
        reporter: Arc<Mutex<ProgressReporter>>,
    ) -> Result<RemoteFile, Error> {
        // Create file data, generate a key
        let file = FileData::from(&self.path)?;
        let key = KeySet::generate(true);

        // Create metadata and a file reader
        let metadata = self.create_metadata(&key, &file)?;
        let reader = self.create_reader(&key, reporter.clone())?;
        let reader_len = reader.len().unwrap();

        // Create the request to send
        let req = self.create_request(
            client,
            &key,
            metadata,
            reader,
        );

        // Start the reporter
        reporter.lock()
            .map_err(|_| UploadError::Progress)?
            .start(reader_len);

        // Execute the request
        // TODO: don't fail on nonce error, just don't use it
        let (result, nonce) = self.execute_request(req, client, &key)?;

        // Mark the reporter as finished
        reporter.lock()
            .map_err(|_| UploadError::Progress)?
            .finish();

        // Change the password if set
        if let Some(password) = self.password {
            Password::new(&result, &password, nonce.clone()).invoke(client)?;
        }

        // Change parameters if set
        if let Some(params) = self.params {
            Params::new(&result, params, nonce.clone()).invoke(client)?;
        }

        Ok(result)
    }

    /// Create a blob of encrypted metadata.
    fn create_metadata(&self, key: &KeySet, file: &FileData)
        -> Result<Vec<u8>, MetaError>
    {
        // Determine what filename to use
        let name = self.name.clone()
            .unwrap_or(file.name().to_owned());

        // Construct the metadata
        let metadata = Metadata::from(
            key.iv(),
            name,
            file.mime().clone(),
        ).to_json().into_bytes();

        // Encrypt the metadata
        let mut metadata_tag = vec![0u8; 16];
        let mut metadata = match encrypt_aead(
            KeySet::cipher(),
            key.meta_key().unwrap(),
            Some(&[0u8; 12]),
            &[],
            &metadata,
            &mut metadata_tag,
        ) {
            Ok(metadata) => metadata,
            Err(_) => return Err(MetaError::Encrypt),
        };

        // Append the encryption tag
        metadata.append(&mut metadata_tag);

        Ok(metadata)
    }

    /// Create a reader that reads the file as encrypted stream.
    fn create_reader(
        &self,
        key: &KeySet,
        reporter: Arc<Mutex<ProgressReporter>>,
    ) -> Result<EncryptedReader, Error> {
        // Open the file
        let file = match File::open(self.path.as_path()) {
            Ok(file) => file,
            Err(err) => return Err(FileError::Open(err).into()),
        };

        // Create an encrypted reader
        let reader = match EncryptedFileReader::new(
            file,
            KeySet::cipher(),
            key.file_key().unwrap(),
            key.iv(),
        ) {
            Ok(reader) => reader,
            Err(_) => return Err(ReaderError::Encrypt.into()),
        };

        // Buffer the encrypted reader
        let reader = BufReader::new(reader);

        // Wrap into the encrypted reader
        let mut reader = ProgressReader::new(reader)
            .map_err(|_| ReaderError::Progress)?;

        // Initialize and attach the reporter
        reader.set_reporter(reporter);

        Ok(reader)
    }

    /// Build the request that will be send to the server.
    fn create_request(
        &self,
        client: &Client,
        key: &KeySet,
        metadata: Vec<u8>,
        reader: EncryptedReader,
    ) -> Request {
        // Get the reader length
        let len = reader.len().expect("failed to get reader length");

        // Configure a form to send
        let part = Part::reader_with_length(reader, len)
            // TODO: keep this here? .file_name(file.name())
            .mime(APPLICATION_OCTET_STREAM);
        let form = Form::new()
            .part("data", part);

        // Define the URL to call
        // TODO: create an error for this unwrap
        let url = self.host.join("api/upload")
            .expect("invalid host");

        // Build the request
        client.post(url.as_str())
            .header(Authorization(
                format!("send-v1 {}", key.auth_key_encoded().unwrap())
            ))
            .header(XFileMetadata::from(&metadata))
            .multipart(form)
            .build()
            .expect("failed to build an API request")
    }

    /// Execute the given request, and create a file object that represents the
    /// uploaded file.
    fn execute_request(&self, req: Request, client: &Client, key: &KeySet) 
        -> Result<(RemoteFile, Option<Vec<u8>>), UploadError>
    {
        // Execute the request
        let mut response = match client.execute(req) {
            Ok(response) => response,
            // TODO: attach the error context
            Err(_) => return Err(UploadError::Request),
        };

        // Validate the status code
        let status = response.status();
        if !status.is_success() {
            return Err(
                UploadError::RequestStatus(status, status.err_text())
            );
        }

        // Try to get the nonce, don't error on failure
        let nonce = response.headers()
            .get_raw(HEADER_AUTH_NONCE)
            .and_then(|h| h.one())
            .and_then(|line| String::from_utf8(line.to_vec()).ok())
            .and_then(|line| line.split_terminator(" ").skip(1).next()
                .map(|line| line.to_owned())
            )
            .and_then(|nonce| b64::decode(&nonce).ok());

        // Decode the response
        let response: UploadResponse = match response.json() {
            Ok(response) => response,
            Err(err) => return Err(UploadError::Decode(err)),
        };

        // Transform the responce into a file object
        Ok((
            response.into_file(self.host.clone(), &key)?,
            nonce,
        ))
    }
}

/// The response from the server after a file has been uploaded.
/// This response contains the file ID and owner key, to manage the file.
///
/// It also contains the download URL, although an additional secret is
/// required.
///
/// The download URL can be generated using `download_url()` which will
/// include the required secret in the URL.
#[derive(Debug, Deserialize)]
struct UploadResponse {
    /// The file ID.
    id: String,

    /// The URL the file is reachable at.
    /// This includes the file ID, but does not include the secret.
    url: String,

    /// The owner key, used to do further file modifications.
    owner: String,
}

impl UploadResponse {
    /// Convert this response into a file object.
    ///
    /// The `host` and `key` must be given.
    pub fn into_file(self, host: Url, key: &KeySet)
        -> Result<RemoteFile, UploadError>
    {
        Ok(
            RemoteFile::new_now(
                self.id,
                host,
                Url::parse(&self.url)?,
                key.secret().to_vec(),
                Some(self.owner),
            )
        )
    }
}

/// A struct that holds various file properties, such as it's name and it's
/// mime type.
struct FileData<'a> {
    /// The file name.
    name: &'a str,

    /// The file mime type.
    mime: Mime,
}

impl<'a> FileData<'a> {
    /// Create a file data object, from the file at the given path.
    pub fn from(path: &'a PathBuf) -> Result<Self, FileError> {
        // Make sure the given path is a file
        if !path.is_file() {
            return Err(FileError::NotAFile);
        }

        // Get the file name
        let name = match path.file_name() {
            Some(name) => name.to_str().unwrap_or("file"),
            None => "file",
        };

        Ok(
            Self {
                name,
                mime: guess_mime_type(path),
            }
        )
    }

    /// Get the file name.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Get the file mime type.
    pub fn mime(&self) -> &Mime {
        &self.mime
    }
}

#[derive(Fail, Debug)]
pub enum Error {
    /// An error occurred while preparing a file for uploading.
    #[fail(display = "Failed to prepare uploading the file")]
    Prepare(#[cause] PrepareError),

    /// An error occurred while opening, reading or using the file that
    /// the should be uploaded.
    // TODO: maybe append the file path here for further information
    #[fail(display = "")]
    File(#[cause] FileError),

    /// An error occurred while uploading the file.
    #[fail(display = "Failed to upload the file")]
    Upload(#[cause] UploadError),

    /// An error occurred while chaining file parameters.
    #[fail(display = "Failed to change file parameters")]
    Params(#[cause] ParamsError),

    /// An error occurred while setting the password.
    #[fail(display = "Failed to set the password")]
    Password(#[cause] PasswordError),
}

impl From<MetaError> for Error {
    fn from(err: MetaError) -> Error {
        Error::Prepare(PrepareError::Meta(err))
    }
}

impl From<FileError> for Error {
    fn from(err: FileError) -> Error {
        Error::File(err)
    }
}

impl From<ReaderError> for Error {
    fn from(err: ReaderError) -> Error {
        Error::Prepare(PrepareError::Reader(err))
    }
}

impl From<UploadError> for Error {
    fn from(err: UploadError) -> Error {
        Error::Upload(err)
    }
}

impl From<ParamsError> for Error {
    fn from(err: ParamsError) -> Error {
        Error::Params(err)
    }
}

impl From<PasswordError> for Error {
    fn from(err: PasswordError) -> Error {
        Error::Password(err)
    }
}

#[derive(Fail, Debug)]
pub enum PrepareError {
    /// Failed to prepare the file metadata for uploading.
    #[fail(display = "Failed to prepare file metadata")]
    Meta(#[cause] MetaError),

    /// Failed to create an encrypted file reader, that encrypts
    /// the file on the fly when it is read.
    #[fail(display = "Failed to access the file to upload")]
    Reader(#[cause] ReaderError),
}

#[derive(Fail, Debug)]
pub enum MetaError {
    /// An error occurred while encrypting the file metadata.
    #[fail(display = "Failed to encrypt file metadata")]
    Encrypt,
}

#[derive(Fail, Debug)]
pub enum ReaderError {
    /// An error occurred while creating the file encryptor.
    #[fail(display = "Failed to create file encryptor")]
    Encrypt,

    /// Failed to create the progress reader, attached to the file reader,
    /// to measure the uploading progress.
    #[fail(display = "Failed to create progress reader")]
    Progress,
}

#[derive(Fail, Debug)]
pub enum FileError {
    /// The given path, is not not a file or doesn't exist.
    #[fail(display = "The given path is not an existing file")]
    NotAFile,

    /// Failed to open the file that must be uploaded for reading.
    #[fail(display = "Failed to open the file to upload")]
    Open(#[cause] IoError),
}

#[derive(Fail, Debug)]
pub enum UploadError {
    /// Failed to start or update the uploading progress, because of this the
    /// upload can't continue.
    #[fail(display = "Failed to update upload progress")]
    Progress,

    /// Sending the request to upload the file failed.
    #[fail(display = "Failed to request file upload")]
    Request,

    /// The response for downloading the indicated an error and wasn't successful.
    #[fail(display = "Bad HTTP response '{}' while requesting file upload", _1)]
    RequestStatus(StatusCode, String),

    /// Failed to decode the upload response from the server.
    /// Maybe the server responded with data from a newer API version.
    #[fail(display = "Failed to decode upload response")]
    Decode(#[cause] ReqwestError),

    /// Failed to parse the retrieved URL from the upload response.
    #[fail(display = "Failed to parse received URL")]
    ParseUrl(#[cause] UrlParseError),
}

impl From<UrlParseError> for UploadError {
    fn from(err: UrlParseError) -> UploadError {
        UploadError::ParseUrl(err)
    }
}
