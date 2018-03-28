use openssl::error::ErrorStack;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Signer;

use super::b64;

/// Compute the signature for the given data and key.
/// This is done using an HMAC key using the SHA256 digest.
///
/// If computing the signature failed, an error is returned.
pub fn signature(key: &[u8], data: &[u8]) -> Result<Vec<u8>, ErrorStack> {
    // Build the key, and signer
    let pkey = PKey::hmac(&key)?;
    let mut signer = Signer::new(MessageDigest::sha256(), &pkey)?;

    // Feed the data
    signer.update(&data)?;

    // Compute the signature
    Ok(signer.sign_to_vec()?)
}

/// Compute the signature for the given data and key.
/// This is done using an HMAC key using the SHA256 digest.
///
/// The resulting signature is encoded as base64 string in an URL-safe manner.
///
/// If computing the signature failed, an error is returned.
pub fn signature_encoded(key: &[u8], data: &[u8])
    -> Result<String, ErrorStack>
{
    signature(key, data).map(|sig| b64::encode(&sig))
}
