extern crate base64;
extern crate clap;
extern crate crypto;
extern crate hyper;
extern crate mime_guess;
extern crate open;
extern crate rand;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::fmt;
use std::fs::File;
use std::io::{self, Cursor, Read};
use std::path::Path;

use clap::{App, Arg};
use crypto::aead::AeadEncryptor;
use crypto::aes::KeySize;
use crypto::aes_gcm::AesGcm;
use crypto::digest::Digest;
use crypto::hkdf::{hkdf_extract, hkdf_expand};
use crypto::sha2::Sha256;
use hyper::error::Error as HyperError;
use mime_guess::Mime;
use rand::{Rng, thread_rng};
use reqwest::header::{
    Authorization,
    Formatter as HeaderFormatter,
    Header,
    Raw
};
use reqwest::mime::APPLICATION_OCTET_STREAM;
use reqwest::multipart::Part;

const TAG_LEN: usize = 16;

fn main() {
    // Handle CLI arguments
    let matches = App::new("ffsend")
        .version("0.1.0")
        .author("Tim Visee <timvisee@gmail.com>")
        .about("A simple Firefox Send CLI client")
        .arg(Arg::with_name("file")
             .short("f")
             .long("file")
             .value_name("PATH")
             .help("The file to upload")
             .required(true)
             .multiple(false))
        .get_matches();

    // Get the path
    let path = Path::new(matches.value_of("file").unwrap());

    // Make sure the path is a file
    if !path.is_file() {
        panic!("The selected path is not a file");
    }

    // TODO: a fixed path for now, as upload test
    let file_ext = path.extension().unwrap().to_str().unwrap();
    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();

    // Create a new reqwest client
    let client = reqwest::Client::new();

    // Generate a secret and iv
    let mut secret = [0u8; 16];
    let mut iv = [0u8; 12];
    thread_rng().fill_bytes(&mut secret);
    thread_rng().fill_bytes(&mut iv);

    // Derive keys
    let encrypt_key = derive_file_key(&secret);
    let auth_key = derive_auth_key(&secret, None, None);
    let meta_key = derive_meta_key(&secret);

    // Generate a file and meta cipher
    // TODO: use the proper key size here, and the proper aad
    let file_cipher = AesGcm::new(KeySize::KeySize128, &encrypt_key, &iv, b"");
    let mut meta_cipher = AesGcm::new(KeySize::KeySize128, &meta_key, &[0u8; 12], b"");

    // Guess the mimetype of the file
    let file_mime = mime_guess::get_mime_type(file_ext);

    // Construct the metadata
    let metadata = Metadata::from(&iv, file_name.clone(), file_mime);

    // Encrypt the metadata, append the tag
    let metadata = metadata.to_json().into_bytes();
    let mut metadata_tag = vec![0u8; 16];
    let mut metadata_encrypted = vec![0u8; metadata.len()];
    meta_cipher.encrypt(&metadata, &mut metadata_encrypted, &mut metadata_tag);
    metadata_encrypted.append(&mut metadata_tag);

    // Open the file and create an encrypted file reader
    let file = File::open(path).unwrap();
    let reader = EncryptedFileReaderTagged::new(file, file_cipher);

    // Build the file part, configure the form to send
    let part = Part::reader(reader)
        .file_name(file_name)
        .mime(APPLICATION_OCTET_STREAM);
    let form = reqwest::multipart::Form::new()
        .part("data", part);

    // Make the request
    let mut res = client.post("http://localhost:8080/api/upload")
        .header(Authorization(format!("send-v1 {}", base64_encode(&auth_key))))
        .header(XFileMetadata::from(&metadata_encrypted))
        .multipart(form)
        .send()
        .unwrap();

    // Parse the response
    let upload_res: UploadResponse = res.json().unwrap();

    // Print the response
    let url = upload_res.download_url(&secret);
    println!("Response: {:#?}", upload_res);
    println!("Secret key: {}", base64_encode(&secret));
    println!("Download URL: {}", url);

    // Open the URL in the browser
    open::that(url);
}

/// Run HKDF crypto.
///
/// # Arguments
/// * length - Length of the derived key value that is returned.
/// * ikm - The input keying material.
/// * info - Optional context and application specific information to use.
///
/// # Returns
/// The output keying material, with the length as as specified in the `length`
/// argument.
fn hkdf<'a>(
    length: usize,
    ikm: &[u8],
    info: Option<&[u8]>
) -> Vec<u8> {
    // Unwrap info or use empty info
    let info = info.unwrap_or(b"");

    // Construct the digest to use
    let digest = Sha256::new();

    // Invoke HKDF extract, create a pseudo random key
    let mut pkr: Vec<u8> = vec![0u8; digest.output_bytes()];
    hkdf_extract(digest, b"", ikm, &mut pkr);

    // Invoke HKDF expand, create the output keying material
    let mut okm: Vec<u8> = vec![0u8; length];
    hkdf_expand(digest, &pkr, info, &mut okm);

    okm
}

fn derive_file_key(secret: &[u8]) -> Vec<u8> {
    hkdf(16, secret, Some(b"encryption"))
}

fn derive_auth_key(secret: &[u8], password: Option<String>, url: Option<String>) -> Vec<u8> {
    if password.is_none() {
        hkdf(64, secret, Some(b"authentication"))
    } else {
        // TODO: implement this
        unimplemented!();
    }
}

fn derive_meta_key(secret: &[u8]) -> Vec<u8> {
    hkdf(16, secret, Some(b"metadata"))
}

/// File metadata, which is send to the server.
#[derive(Serialize)]
struct Metadata {
    /// The input vector.
    iv: String,

    /// The file name.
    name: String,

    /// The file mimetype.
    #[serde(rename="type")]
    mime: String,
}

impl Metadata {
    /// Construct metadata from the given properties.
    ///
    /// Parameters:
    /// * iv: initialisation vector
    /// * name: file name
    /// * mime: file mimetype
    pub fn from(iv: &[u8], name: String, mime: Mime) -> Self {
        Metadata {
            iv: base64_encode(iv),
            name,
            mime: mime.to_string(),
        }
    }

    /// Convert this structure to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

/// A X-File-Metadata header for reqwest, that is used to pass encrypted
/// metadata to the server.
///
/// The encrypted metadata (bytes) is base64 encoded when constructing this
/// header using `from`.
#[derive(Clone)]
struct XFileMetadata {
    /// The metadata, as a base64 encoded string.
    metadata: String,
}

impl XFileMetadata {
    /// Construct the header from the given encrypted metadata.
    pub fn from(bytes: &[u8]) -> Self {
        XFileMetadata {
            metadata: base64_encode(bytes),
        }
    }
}

impl Header for XFileMetadata {
    fn header_name() -> &'static str {
        "X-File-Metadata"
    }

    fn parse_header(_raw: &Raw) -> Result<Self, HyperError> {
        // TODO: implement this some time
        unimplemented!();
    }

    fn fmt_header(&self, f: &mut HeaderFormatter) -> fmt::Result {
        // TODO: is this encoding base64 for us?
        f.fmt_line(&self.metadata)
    }
}

/// A file reader, that encrypts the file that is read with the given
/// `cipher`, and appends the cipher tag to the end of it.
///
/// This reader is not lazy, and loads the whole file in memory to
/// encrypt it at once. Also, a buffer is created to copy the encrypted file
/// into.
///
/// This object requires about twice the memory as the size of the file that is
/// encrypted when constructed.
struct EncryptedFileReaderTagged {
    /// A cursor that reads encrypted file data.
    data: Cursor<Vec<u8>>,

    /// A tag cursor that reads the tag to append.
    tag: Cursor<Vec<u8>>,
}

impl EncryptedFileReaderTagged {
    /// Construct a new reader for the given `file` with the given `cipher`.
    ///
    /// This method consumes twice the size of the file in memory while
    /// constructing, and constructs a reader that has a size similar to the
    /// file.
    pub fn new(mut file: File, mut cipher: AesGcm<'static>) -> Self {
        // Get the length of the file
        let len = file.metadata().unwrap().len() as usize;

        // Read the whole file in a data buffer
        let mut data = Vec::with_capacity(len);
        file.read_to_end(&mut data).unwrap();

        // Create an encrypted and tag buffer
        let mut encrypted = vec![0u8; data.len()];
        let mut tag = vec![0u8; TAG_LEN];

        // Encrypt the file, set the tag
        cipher.encrypt(&data, &mut encrypted, &mut tag);

        // Construct the reader and return
        EncryptedFileReaderTagged {
            data: Cursor::new(encrypted),
            tag: Cursor::new(tag),
        }
    }
}

impl Read for EncryptedFileReaderTagged {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, io::Error> {
        // Read the data if we haven't started with the tag yet
        if self.tag.position() == 0 {
            // Read and return if something was read
            let result = self.data.read(buf);
            match result {
                Ok(len) if len > 0 => return result,
                _ => {},
            }
        }

        // Read the tag if it's empty
        self.tag.read(buf)
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
    /// unkhe URL the file is reachable at.
    /// This includes the file ID, but does not include the secret.
    url: String,

    /// The owner key, used to do further file modifications.
    owner: String,

    /// The file ID.
    id: String,
}

impl UploadResponse {
    /// Get the download URL, including the secret.
    ///
    /// The secret bytes must be passed to `secret`.
    pub fn download_url(&self, secret: &[u8]) -> String {
        format!("{}#{}", self.url, base64_encode(secret))
    }
}

/// Encode the given byte slice using base64, in an URL-safe manner.
fn base64_encode(input: &[u8]) -> String {
    base64::encode_config(input, base64::URL_SAFE_NO_PAD)
}
