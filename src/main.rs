extern crate clap;
extern crate hyper;
extern crate mime_guess;
extern crate open;
extern crate openssl;
extern crate rand;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;

mod b64;
mod crypto;
mod metadata;
mod reader;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use clap::{App, Arg};
use openssl::symm::{Cipher, encrypt_aead};
use rand::{Rng, thread_rng};
use reqwest::header::Authorization;
use reqwest::mime::APPLICATION_OCTET_STREAM;
use reqwest::multipart::Part;

use crypto::{derive_auth_key, derive_file_key, derive_meta_key};
use metadata::{Metadata, XFileMetadata};
use reader::EncryptedFileReaderTagged;

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

    // Guess the mimetype of the file
    let file_mime = mime_guess::get_mime_type(file_ext);

    // Construct the metadata
    let metadata = Metadata::from(&iv, file_name.clone(), file_mime);

    // Convert the metadata to JSON bytes
    let metadata = metadata.to_json().into_bytes();

    // Choose a file and meta cipher type
    let cipher = Cipher::aes_128_gcm();

    // Encrypt the metadata, and append the tag to it
    let mut metadata_tag = vec![0u8; 16];
    let mut metadata = encrypt_aead(
        cipher,
        &meta_key,
        Some(&[0u8; 12]),
        &[],
        &metadata,
        &mut metadata_tag,
    ).unwrap();
    metadata.append(&mut metadata_tag);

    // Open the file and create an encrypted file reader
    let file = File::open(path).unwrap();
    let reader = EncryptedFileReaderTagged::new(
        file,
        cipher,
        &encrypt_key,
        &iv,
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

    // Make the request
    let mut res = client.post("http://localhost:8080/api/upload")
        .header(Authorization(format!("send-v1 {}", b64::encode(&auth_key))))
        .header(XFileMetadata::from(&metadata))
        .multipart(form)
        .send()
        .unwrap();

    // Parse the response
    let upload_res: UploadResponse = res.json().unwrap();

    // Print the response
    let url = upload_res.download_url(&secret);
    println!("Response: {:#?}", upload_res);
    println!("Secret key: {}", b64::encode(&secret));
    println!("Download URL: {}", url);

    // Open the URL in the browser
    open::that(url).expect("failed to open URL");
}

// TODO: implement this some other way
unsafe impl Send for EncryptedFileReaderTagged {}

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
        format!("{}#{}", self.url, b64::encode(secret))
    }
}
