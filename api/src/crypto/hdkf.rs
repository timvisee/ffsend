extern crate hkdf;
extern crate sha2;

use self::hkdf::Hkdf;
use openssl::hash::MessageDigest;
use openssl::pkcs5::pbkdf2_hmac;
use self::sha2::Sha256;
use url::Url;

/// The length of the derived authentication key in bytes.
const KEY_AUTH_SIZE: usize = 64;

/// The number of iterations to do for deriving a passworded authentication
/// key.
const KEY_AUTH_ITERATIONS: usize = 100;

/// Derive a HKDF key.
///
/// No _salt_ bytes are used in this function.
///
/// # Arguments
/// * length - Length of the derived key value that is returned.
/// * ikm - The input keying material.
/// * info - Optional context and application specific information to use.
///
/// # Returns
/// The output keying material, with the length as as specified in the `length`
/// argument.
fn hkdf(
    length: usize,
    ikm: &[u8],
    info: Option<&[u8]>,
) -> Vec<u8> {
    // Unwrap info or use empty info
    let info = info.unwrap_or(&[]);

    // Derive a HKDF key with the given length
    Hkdf::<Sha256>::new(&ikm, &[])
        .derive(&info, length)
}

/// Derive a key to use for file data encryption, based on the given `secret`.
pub fn derive_file_key(secret: &[u8]) -> Vec<u8> {
    hkdf(16, secret, Some(b"encryption"))
}

/// Derive a key to use for metadata encryption, based on the given `secret`.
pub fn derive_meta_key(secret: &[u8]) -> Vec<u8> {
    hkdf(16, secret, Some(b"metadata"))
}

/// Derive a key used for authentication, based on the given `secret`.
///
/// A `password` and `url` may be given for special key deriving.
/// At this time this is not implemented however.
pub fn derive_auth_key(secret: &[u8], password: Option<&str>, url: Option<&Url>) -> Vec<u8> {
    // Nothing, or both a password and URL must be given
    assert_eq!(
        password.is_none(),
        url.is_none(),
        "unable to derive authentication key, missing password or URL",
    );

    // Derive a key without a password
    if password.is_none() {
        return hkdf(KEY_AUTH_SIZE, secret, Some(b"authentication"));
    }

    // Derive a key with a password and URL
    // TODO: do not expect/unwrap here
    let mut key = vec![0u8; KEY_AUTH_SIZE];
    pbkdf2_hmac(
        password.unwrap().as_bytes(),
        url.unwrap().as_str().as_bytes(),
        KEY_AUTH_ITERATIONS,
        MessageDigest::sha256(),
        &mut key,
    ).expect("failed to derive passworded authentication key");

    key
}
