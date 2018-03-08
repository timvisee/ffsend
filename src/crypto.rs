extern crate hkdf;
extern crate sha2;

use self::hkdf::Hkdf;
use self::sha2::Sha256;

// Reexport the cryptographically secure random bytes generator
pub use super::openssl::rand::rand_bytes;

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
fn hkdf<'a>(
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
pub fn derive_auth_key(secret: &[u8], password: Option<String>, _url: Option<String>) -> Vec<u8> {
    if password.is_none() {
        hkdf(64, secret, Some(b"authentication"))
    } else {
        // TODO: implement this
        unimplemented!();
    }
}
