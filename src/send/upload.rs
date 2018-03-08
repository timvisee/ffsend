use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use super::super::mime_guess::get_mime_type;
use super::super::openssl::symm::encrypt_aead;
use super::super::reqwest;
use super::super::reqwest::header::Authorization;
use super::super::reqwest::mime::APPLICATION_OCTET_STREAM;
use super::super::reqwest::multipart::Part;
use super::super::url::Url;

use super::key_set::KeySet;
use super::metadata::{Metadata, XFileMetadata};
use super::reader::EncryptedFileReaderTagged;
use super::send_file::SendFile;

pub type Result<T> = ::std::result::Result<T, UploadError>;

/// A file upload action to a Send server.
pub struct Upload {
    /// The Send host to upload the file to.
    host: Url,

    /// The file to upload.
    path: Box<Path>,
}

impl Upload {
    /// Construct a new upload action.
    pub fn new(host: Url, path: Box<Path>) -> Self {
        Self {
            host,
            path,
        }
    }

    /// Invoke the upload action.
    pub fn invoke(self) -> Result<SendFile> {
        // Make sure the given path is a file
        if !self.path.is_file() {
            return Err(UploadError::NotAFile);
        }

        // Grab some file details
        let file_ext = self.path.extension().unwrap().to_str().unwrap();
        let file_name = self.path.file_name().unwrap().to_str().unwrap().to_owned();
        let file_mime = get_mime_type(file_ext);

        // Generate a key set
        let key = KeySet::generate(true);

        // Construct the metadata
        let metadata = Metadata::from(key.iv(), file_name.clone(), file_mime)
            .to_json()
            .into_bytes();

        // Encrypt the metadata, and append the tag to it
        let mut metadata_tag = vec![0u8; 16];
        let mut metadata = encrypt_aead(
            KeySet::cipher(),
            key.meta_key().unwrap(),
            Some(&[0u8; 12]),
            &[],
            &metadata,
            &mut metadata_tag,
        ).unwrap();
        metadata.append(&mut metadata_tag);

        // Open the file and create an encrypted file reader
        let file = File::open(&self.path).unwrap();
        let reader = EncryptedFileReaderTagged::new(
            file,
            KeySet::cipher(),
            key.file_key().unwrap(),
            key.iv(),
        ).unwrap();

        // Buffer the encrypted reader, and determine the length
        let reader_len = reader.len().unwrap();
        let reader = BufReader::new(reader);

        // Build the file part, configure the form to send
        let part = Part::reader_with_length(reader, reader_len)
            .file_name(file_name)
            .mime(APPLICATION_OCTET_STREAM);
        let form = reqwest::multipart::Form::new()
            .part("data", part);

        // Create a new reqwest client
        let client = reqwest::Client::new();

        // Make the request
        // TODO: properly format an URL here
        let url = self.host.join("api/upload").expect("invalid host");
        let mut res = client.post(url.as_str())
            .header(Authorization(format!("send-v1 {}", key.auth_key_encoded().unwrap())))
            .header(XFileMetadata::from(&metadata))
            .multipart(form)
            .send()
            .unwrap();

        // Parse the response
        let upload_res: UploadResponse = res.json().unwrap();

        // Print the response
        Ok(
            upload_res.into_file(self.host, key.secret().to_vec())
        )
    }
}

pub enum UploadError {
    /// The given file is not not an existing file.
    /// Maybe it is a directory, or maybe it doesn't exist.
    NotAFile,
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
pub struct UploadResponse {
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
    /// The `host` and `secret` must be given.
    pub fn into_file(self, host: Url, secret: Vec<u8>) -> SendFile {
        SendFile::new_now(
            self.id,
            host,
            self.url,
            secret,
            self.owner,
        )
    }
}
