extern crate base64;
extern crate crypto;
extern crate mime_guess;
extern crate rand;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::path::Path;

use crypto::aead::AeadEncryptor;
use crypto::aes::KeySize;
use crypto::aes_gcm::AesGcm;
use crypto::digest::Digest;
use crypto::hkdf::{hkdf_extract, hkdf_expand};
use crypto::sha2::Sha256;
use mime_guess::Mime;
use rand::{Rng, thread_rng};

fn main() {
    // TODO: a fixed path for now, as upload test
    let path = Path::new("/home/timvisee/Pictures/Avatar/1024x1024/Avatar.png");
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
    let mut file_cipher = AesGcm::new(KeySize::KeySize128, &encrypt_key, &iv, b"");
    let mut meta_cipher = AesGcm::new(KeySize::KeySize128, &meta_key, &[0u8; 12], b"");

    // Guess the mimetype of the file
    let file_mime = mime_guess::get_mime_type(file_ext);

    // Construct the metadata
    let metadata = Metadata::from(&iv, file_name, file_mime);

    // Encrypt the metadata, append the tag
    let metadata = metadata.to_json().into_bytes();
    let mut metadata_tag = vec![0u8; 16];
    let mut metadata_encrypted = vec![0u8; metadata.len()];
    meta_cipher.encrypt(&metadata, &mut metadata_encrypted, &mut metadata_tag);
    metadata_encrypted.append(&mut metadata_tag);

    let form = reqwest::multipart::Form::new()
        .file("data", path)
        .unwrap();

    let mut res = client.post("http://localhost:8080/api/upload")
        .multipart(form)
        .send()
        .unwrap();

    let text = res.text().unwrap();

    // TODO: remove after debugging
    println!("TEXT: {}", text);
}

#[derive(Serialize)]
struct Metadata {
    /// The input vector
    iv: String,

    /// The file name
    name: String,

    /// The file mimetype
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
            iv: base64::encode(iv),
            name,
            mime: mime.to_string(),
        }
    }

    /// Convert this structure to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

fn derive_file_key(secret: &[u8]) -> Vec<u8> {
    hkdf(16, secret, None, Some(b"encryption"))
}

fn derive_auth_key(secret: &[u8], password: Option<String>, url: Option<String>) -> Vec<u8> {
    if password.is_none() {
        hkdf(64, secret, None, Some(b"authentication"))
    } else {
        // TODO: implement this
        unimplemented!();
    }
}

fn derive_meta_key(secret: &[u8]) -> Vec<u8> {
    hkdf(16, secret, None, Some(b"metadata"))
}

fn hkdf<'a>(
    length: usize,
    ikm: &[u8],
    salt: Option<&[u8]>,
    info: Option<&[u8]>
) -> Vec<u8> {
    // Get the salt and info parameters, use defaults if undefined
    let salt = salt.unwrap_or(b"");
    let info = info.unwrap_or(b"");

    // Define the digest to use
    let digest = Sha256::new();

    let mut pkr: Vec<u8> = vec![0u8; digest.output_bytes()];
    hkdf_extract(digest, salt, ikm, &mut pkr);

    let mut okm: Vec<u8> = vec![0u8; length];
    hkdf_expand(digest, &pkr, info, &mut okm);

    okm
}
