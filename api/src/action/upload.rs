use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use mime_guess::{get_mime_type, Mime};
use openssl::symm::encrypt_aead;
use reqwest::{
    Client, 
    Error as ReqwestError,
    Request,
};
use reqwest::header::Authorization;
use reqwest::mime::APPLICATION_OCTET_STREAM;
use reqwest::multipart::{Form, Part};
use url::Url;

use crypto::key_set::KeySet;
use reader::{
    EncryptedFileReaderTagged,
    ExactLengthReader,
    ProgressReader,
    ProgressReporter,
};
use file::file::File as SendFile;
use file::metadata::{Metadata, XFileMetadata};

type EncryptedReader =
    ProgressReader<'static, BufReader<EncryptedFileReaderTagged>>;
pub type Result<T> = ::std::result::Result<T, UploadError>;

/// A file upload action to a Send server.
pub struct Upload {
    /// The Send host to upload the file to.
    host: Url,

    /// The file to upload.
    path: PathBuf,
}

impl Upload {
    /// Construct a new upload action.
    pub fn new(host: Url, path: PathBuf) -> Self {
        Self {
            host,
            path,
        }
    }

    /// Invoke the upload action.
    pub fn invoke(
        self,
        client: &Client,
        reporter: Box<ProgressReporter + 'static>,
    ) -> Result<SendFile> {
        // Create file data, generate a key
        let file = FileData::from(Box::new(&self.path))?;
        let key = KeySet::generate(true);

        // Crpate metadata and a file reader
        let metadata = self.create_metadata(&key, &file)?;
        // TODO: do not use leak, as it might cause memory leaks
        let reader = self.create_reader(&key, Box::leak(reporter))?;

        // Create the request to send
        let req = self.create_request(
            client,
            &key,
            metadata,
            reader,
        );

        // Execute the request
        let result = self.execute_request(req, client, &key);

        // TODO: finish the progress bar

        result
    }

    /// Create a blob of encrypted metadata.
    fn create_metadata(&self, key: &KeySet, file: &FileData)
        -> Result<Vec<u8>>
    {
        // Construct the metadata
        let metadata = Metadata::from(
            key.iv(),
            file.name().to_owned(),
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
            Err(_) => return Err(UploadError::EncryptionError),
        };

        // Append the encryption tag
        metadata.append(&mut metadata_tag);

        Ok(metadata)
    }

    /// Create a reader that reads the file as encrypted stream.
    fn create_reader(
        &self,
        key: &KeySet,
        reporter: &'static mut ProgressReporter,
    ) -> Result<EncryptedReader> {
        // Open the file
        let file = match File::open(self.path.as_path()) {
            Ok(file) => file,
            Err(_) => return Err(UploadError::FileError),
        };

        // Create an encrypted reader
        let reader = match EncryptedFileReaderTagged::new(
            file,
            KeySet::cipher(),
            key.file_key().unwrap(),
            key.iv(),
        ) {
            Ok(reader) => reader,
            Err(_) => return Err(UploadError::EncryptionError),
        };

        // Buffer the encrypted reader
        let reader = BufReader::new(reader);

        // Wrap into the encrypted reader
        let mut reader = ProgressReader::new(reader)
            .expect("failed to create progress reader");

        // Initialize and attach the reporter
        reporter.start(reader.len().unwrap());
        reader.set_reporter(&mut *reporter);

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
            // .file_name(file.name())
            .mime(APPLICATION_OCTET_STREAM);
        let form = Form::new()
            .part("data", part);

        // Define the URL to call
        let url = self.host.join("api/upload").expect("invalid host");

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
        -> Result<SendFile>
    {
        // Execute the request
        let mut res = match client.execute(req) {
            Ok(res) => res,
            Err(err) => return Err(UploadError::RequestError(err)),
        };

        // Decode the response
        let res: UploadResponse = match res.json() {
            Ok(res) => res,
            Err(_) => return Err(UploadError::DecodeError),
        };

        // Transform the responce into a file object
        Ok(res.into_file(self.host.clone(), &key))
    }
}

/// Errors that may occur in the upload action. 
#[derive(Debug)]
pub enum UploadError {
    /// The given file is not not an existing file.
    /// Maybe it is a directory, or maybe it doesn't exist.
    NotAFile,

    /// An error occurred while opening or reading a file.
    FileError,

    /// An error occurred while encrypting the file.
    EncryptionError,

    /// An error occurred while while processing the request.
    /// This also covers things like HTTP 404 errors.
    RequestError(ReqwestError),

    /// An error occurred while decoding the response data.
    DecodeError,
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
    pub fn into_file(self, host: Url, key: &KeySet) -> SendFile {
        SendFile::new_now(
            self.id,
            host,
            Url::parse(&self.url)
                .expect("upload response URL parse error"),
            key.secret().to_vec(),
            self.owner,
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
    pub fn from(path: Box<&'a Path>) -> Result<Self> {
        // Make sure the given path is a file
        if !path.is_file() {
            return Err(UploadError::NotAFile);
        }

        // Get the file name
        let name = match path.file_name() {
            Some(name) => name.to_str().expect("failed to convert string"),
            None => return Err(UploadError::FileError),
        };

        // Get the file extention
        // TODO: handle cases where the file doesn't have an extention
        let ext = match path.extension() {
            Some(ext) => ext.to_str().expect("failed to convert string"),
            None => return Err(UploadError::FileError),
        };

        Ok(
            Self {
                name,
                mime: get_mime_type(ext),
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
