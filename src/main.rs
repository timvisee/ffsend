extern crate hyper;
extern crate lazy_static;
extern crate mime_guess;
extern crate open;
extern crate openssl;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate url;

mod action;
mod app;
mod b64;
mod cmd;
mod crypto;
mod metadata;
mod reader;
mod send;
mod util;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use openssl::symm::{Cipher, encrypt_aead};
use reqwest::header::Authorization;
use reqwest::mime::APPLICATION_OCTET_STREAM;
use reqwest::multipart::Part;

use action::upload::UploadResponse;
use cmd::Handler;
use cmd::cmd_upload::CmdUpload;
use metadata::{Metadata, XFileMetadata};
use reader::EncryptedFileReaderTagged;
use send::key_set::KeySet;

/// Application entrypoint.
fn main() {
    // Parse CLI arguments
    let cmd_handler = Handler::parse();

    // Invoke the proper action
    invoke_action(&cmd_handler);
}

/// Invoke the proper action based on the CLI input.
///
/// If no proper action is selected, the program will quit with an error
/// message.
fn invoke_action(handler: &Handler) {
    // Match the upload command
    if let Some(cmd) = handler.upload() {
        return action_upload(&cmd);
    }

    // No subcommand was selected, show general help
    Handler::build()
        .print_help()
        .expect("failed to print command help");
}

/// The upload action.
fn action_upload(cmd_upload: &CmdUpload) {
    // Get the path and host
    let path = Path::new(cmd_upload.file());
    let host = cmd_upload.host();

    // Make sure the path is a file
    if !path.is_file() {
        panic!("The selected path is not a file");
    }

    // TODO: a fixed path for now, as upload test
    let file_ext = path.extension().unwrap().to_str().unwrap();
    let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();

    // Create a new reqwest client
    let client = reqwest::Client::new();

    // Generate a key
    let key = KeySet::generate(true);

    // Guess the mimetype of the file
    let file_mime = mime_guess::get_mime_type(file_ext);

    // Construct the metadata
    let metadata = Metadata::from(key.iv(), file_name.clone(), file_mime);

    // Convert the metadata to JSON bytes
    let metadata = metadata.to_json().into_bytes();

    // Choose a file and meta cipher type
    let cipher = Cipher::aes_128_gcm();

    // Encrypt the metadata, and append the tag to it
    let mut metadata_tag = vec![0u8; 16];
    let mut metadata = encrypt_aead(
        cipher,
        key.meta_key().unwrap(),
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

    // Make the request
    // TODO: properly format an URL here
    let url = host.join("api/upload").expect("invalid host");
    let mut res = client.post(url.as_str())
        .header(Authorization(format!("send-v1 {}", key.auth_key_encoded().unwrap())))
        .header(XFileMetadata::from(&metadata))
        .multipart(form)
        .send()
        .unwrap();

    // Parse the response
    let upload_res: UploadResponse = res.json().unwrap();

    // Print the response
    let file = upload_res.into_file(host, key.secret().to_vec());
    let url = file.download_url();
    println!("File: {:#?}", file);
    println!("Secret key: {}", key.secret_encoded());
    println!("Download URL: {}", url);

    // Open the URL in the browser
    open::that(url).expect("failed to open URL");
}

// TODO: implement this some other way
unsafe impl Send for EncryptedFileReaderTagged {}
