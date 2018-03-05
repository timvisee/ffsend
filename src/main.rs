extern crate crypto;
extern crate rand;
extern crate reqwest;

//use bytes::BytesMut;
use crypto::aes::KeySize;
use crypto::aes_gcm::AesGcm;
use crypto::digest::Digest;
use crypto::hkdf::{hkdf_extract, hkdf_expand};
use crypto::sha2::Sha256;
use rand::{Rng, thread_rng};

fn main() {
    // TODO: a fixed path for now, as upload test
    let path = "/home/timvisee/Pictures/Avatar/1024x1024/Avatar.png";

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
    let mut meta_cipher = AesGcm::new(KeySize::KeySize128, &encrypt_key, &[0u8; 12], b"");

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
    let mut digest = Sha256::new();

    let mut pkr: Vec<u8> = vec![0u8; digest.output_bytes()];
    hkdf_extract(digest, salt, ikm, &mut pkr);

    let mut okm: Vec<u8> = vec![0u8; length];
    hkdf_expand(digest, &pkr, info, &mut okm);

    okm
}
