use std::fs::File;
use std::io;

use openssl::symm::decrypt_aead;
use reqwest::{
    Client, 
    Error as ReqwestError,
};
use reqwest::header::Authorization;
use reqwest::header::ContentLength;
use serde_json;

use crypto::b64;
use crypto::key_set::KeySet;
use crypto::sign::signature_encoded;
use file::file::DownloadFile;
use file::metadata::Metadata;
use reader::EncryptedFileWriter;

pub type Result<T> = ::std::result::Result<T, DownloadError>;

/// The name of the header that is used for the authentication nonce.
const HEADER_AUTH_NONCE: &'static str = "WWW-Authenticate";

// TODO: experiment with `iv` of `None` in decrypt logic

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
    ) -> Result<()> {
        // Create a key set for the file
        let mut key = KeySet::from(self.file);

        // Build the meta cipher
        // let mut metadata_tag = vec![0u8; 16];
        // let mut meta_cipher = match encrypt_aead(
        //     KeySet::cipher(),
        //     self.meta_key().unwrap(),
        //     self.iv,
        //     &[],
        //     &metadata,
        //     &mut metadata_tag,
        // ) {
        //     Ok(cipher) => cipher,
        //     Err(_) => // TODO: return error here,
        // };

        // Get the download url, and parse the nonce
        // TODO: do not unwrap here, return error
        let download_url = self.file.download_url(false);
        let response = client.get(download_url)
            .send()
            .expect("failed to get nonce, failed to send file request");

        // Validate the status code
        // TODO: allow redirects here?
        if !response.status().is_success() {
            // TODO: return error here
            panic!("failed to get nonce, request status is not successful");
        }

        // Get the authentication nonce
        // TODO: don't unwrap here, return an error
        let nonce = b64::decode_standard(
            response.headers()
                .get_raw(HEADER_AUTH_NONCE)
                .expect("missing authenticate header") 
                .one()
                .map(|line| String::from_utf8(line.to_vec())
                    .expect("invalid authentication header contents")
                )
                .expect("authentication header is empty")
                .split_terminator(" ")
                .skip(1)
                .next()
                .expect("missing authentication nonce")
        ).expect("failed to decode authentication nonce");

        // Compute the cryptographic signature
        // TODO: do not unwrap, return an error
        let sig = signature_encoded(key.auth_key().unwrap(), &nonce)
            .expect("failed to compute metadata signature");

        // Get the meta URL, fetch the metadata
        // TODO: do not unwrap here, return error
        let meta_url = self.file.api_meta_url();
        let mut response = client.get(meta_url)
            .header(Authorization(
                format!("send-v1 {}", sig)
            ))
            .send()
            .expect("failed to fetch metadata, failed to send request");

        // Validate the status code
        // TODO: allow redirects here?
        if !response.status().is_success() {
            // TODO: return error here
            panic!("failed to fetch metadata, request status is not successful");
        }

        // Get the metadata nonce
        // TODO: don't unwrap here, return an error
        let nonce = b64::decode_standard(
            response.headers()
                .get_raw(HEADER_AUTH_NONCE)
                .expect("missing authenticate header") 
                .one()
                .map(|line| String::from_utf8(line.to_vec())
                    .expect("invalid authentication header contents")
                )
                .expect("authentication header is empty")
                .split_terminator(" ")
                .skip(1)
                .next()
                .expect("missing metadata nonce")
        ).expect("failed to decode metadata nonce");

        // Parse the metadata response
        let meta_response: MetadataResponse = response.json()
            .expect("failed to parse metadata response");

        // Decrypt the metadata, set the input vector
        let metadata = meta_response.decrypt_metadata(&key)
            .expect("failed to decrypt metadata");
        key.set_iv(metadata.iv());

        // Compute the cryptographic signature
        // TODO: do not unwrap, return an error
        let sig = signature_encoded(key.auth_key().unwrap(), &nonce)
            .expect("failed to compute file signature");

        // Get the download URL, build the download request
        // TODO: do not unwrap here, return error
        let download_url = self.file.api_download_url();
        let mut response = client.get(download_url)
            .header(Authorization(
                format!("send-v1 {}", sig)
            ))
            .send()
            .expect("failed to fetch file, failed to send request");

        // Validate the status code
        // TODO: allow redirects here?
        if !response.status().is_success() {
            // TODO: return error here
            panic!("failed to fetch file, request status is not successful");
        }

        // Get the content length
        let response_len = response.headers().get::<ContentLength>()
            .expect("failed to fetch file, missing content length header")
            .0;

        // Open a file to write to
        // TODO: this should become a temporary file first
        let out = File::create("downloaded.toml")
            .expect("failed to open file");
        let mut writer = EncryptedFileWriter::new(
            out,
            response_len as usize,
            KeySet::cipher(),
            key.file_key().unwrap(),
            key.iv(),
        ).expect("failed to create encrypted writer");

        // Write to the output file
        io::copy(&mut response, &mut writer)
            .expect("failed to download and decrypt file");

        // Verify the writer
        // TODO: delete the file if verification failed, show a proper error
        assert!(writer.verified(), "downloaded and decrypted file could not be verified");

        // // Crpate metadata and a file reader
        // let metadata = self.create_metadata(&key, &file)?;
        // let reader = self.create_reader(&key, reporter.clone())?;
        // let reader_len = reader.len().unwrap();

        // // Create the request to send
        // let req = self.create_request(
        //     client,
        //     &key,
        //     metadata,
        //     reader,
        // );

        // // Start the reporter
        // reporter.lock()
        //     .expect("unable to start progress, failed to get lock")
        //     .start(reader_len);

        // // Execute the request
        // let result = self.execute_request(req, client, &key);

        // // Mark the reporter as finished
        // reporter.lock()
        //     .expect("unable to finish progress, failed to get lock")
        //     .finish();
        
        // TODO: return the file path
        // TODO: return the new remote state (does it still exist remote)

        Ok(())
    }

    // /// Create a blob of encrypted metadata.
    // fn create_metadata(&self, key: &KeySet, file: &FileData)
    //     -> Result<Vec<u8>>
    // {
    //     // Construct the metadata
    //     let metadata = Metadata::from(
    //         key.iv(),
    //         file.name().to_owned(),
    //         file.mime().clone(),
    //     ).to_json().into_bytes();

    //     // Encrypt the metadata
    //     let mut metadata_tag = vec![0u8; 16];
    //     let mut metadata = match encrypt_aead(
    //         KeySet::cipher(),
    //         key.meta_key().unwrap(),
    //         Some(&[0u8; 12]),
    //         &[],
    //         &metadata,
    //         &mut metadata_tag,
    //     ) {
    //         Ok(metadata) => metadata,
    //         Err(_) => return Err(DownloadError::EncryptionError),
    //     };

    //     // Append the encryption tag
    //     metadata.append(&mut metadata_tag);

    //     Ok(metadata)
    // }

    // /// Create a reader that reads the file as encrypted stream.
    // fn create_reader(
    //     &self,
    //     key: &KeySet,
    //     reporter: Arc<Mutex<ProgressReporter>>,
    // ) -> Result<EncryptedReader> {
    //     // Open the file
    //     let file = match File::open(self.path.as_path()) {
    //         Ok(file) => file,
    //         Err(_) => return Err(DownloadError::FileError),
    //     };

    //     // Create an encrypted reader
    //     let reader = match EncryptedFileReaderTagged::new(
    //         file,
    //         KeySet::cipher(),
    //         key.file_key().unwrap(),
    //         key.iv(),
    //     ) {
    //         Ok(reader) => reader,
    //         Err(_) => return Err(DownloadError::EncryptionError),
    //     };

    //     // Buffer the encrypted reader
    //     let reader = BufReader::new(reader);

    //     // Wrap into the encrypted reader
    //     let mut reader = ProgressReader::new(reader)
    //         .expect("failed to create progress reader");

    //     // Initialize and attach the reporter
    //     reader.set_reporter(reporter);

    //     Ok(reader)
    // }

    // /// Build the request that will be send to the server.
    // fn create_request(
    //     &self,
    //     client: &Client,
    //     key: &KeySet,
    //     metadata: Vec<u8>,
    //     reader: EncryptedReader,
    // ) -> Request {
    //     // Get the reader length
    //     let len = reader.len().expect("failed to get reader length");

    //     // Configure a form to send
    //     let part = Part::reader_with_length(reader, len)
    //         // .file_name(file.name())
    //         .mime(APPLICATION_OCTET_STREAM);
    //     let form = Form::new()
    //         .part("data", part);

    //     // Define the URL to call
    //     let url = self.host.join("api/upload").expect("invalid host");

    //     // Build the request
    //     client.post(url.as_str())
    //         .header(Authorization(
    //             format!("send-v1 {}", key.auth_key_encoded().unwrap())
    //         ))
    //         .header(XFileMetadata::from(&metadata))
    //         .multipart(form)
    //         .build()
    //         .expect("failed to build an API request")
    // }

    // /// Execute the given request, and create a file object that represents the
    // /// uploaded file.
    // fn execute_request(&self, req: Request, client: &Client, key: &KeySet) 
    //     -> Result<SendFile>
    // {
    //     // Execute the request
    //     let mut res = match client.execute(req) {
    //         Ok(res) => res,
    //         Err(err) => return Err(DownloadError::RequestError(err)),
    //     };

    //     // Decode the response
    //     let res: DownloadResponse = match res.json() {
    //         Ok(res) => res,
    //         Err(_) => return Err(DownloadError::DecodeError),
    //     };

    //     // Transform the responce into a file object
    //     Ok(res.into_file(self.host.clone(), &key))
    // }
}

/// Errors that may occur in the upload action. 
#[derive(Debug)]
pub enum DownloadError {
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
        let raw = b64::decode_url(&self.meta)
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
