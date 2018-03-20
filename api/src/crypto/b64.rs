//! A simple module for encoding or decoding a base64 string from or to a
//! byte array.
//!
//! This module uses an URL-safe scheme, and doesn't add additional padding
//! to the encoded strings.

extern crate base64;

pub use self::base64::{Config, DecodeError};

/// Encode the given byte slice using base64,
/// in an URL-safe manner without padding.
pub fn encode(input: &[u8]) -> String {
    base64::encode_config(input, base64::URL_SAFE_NO_PAD)
}

/// Decode the given string as base64.
/// Standard and URL-safe character sets are both supported,
/// padding is optional.
pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    base64::decode_config(
        input.replace('+', "-")
            .replace('/', "_")
            .trim_right_matches('='),
        base64::URL_SAFE_NO_PAD,
    )
}
