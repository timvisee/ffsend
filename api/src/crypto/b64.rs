//! A simple module for encoding or decoding a base64 string from or to a
//! byte array.
//!
//! This module uses an URL-safe scheme, and doesn't add additional padding
//! to the encoded strings.

extern crate base64;

pub use self::base64::{Config, DecodeError};

/// Encode the given byte slice using base64, in an URL-safe manner.
pub fn encode(input: &[u8]) -> String {
    base64::encode_config(input, base64::URL_SAFE_NO_PAD)
}

/// Decode the given string as base64, with the given configuration.
pub fn decode(input: &str, config: Config) -> Result<Vec<u8>, DecodeError> {
    base64::decode_config(input, config)
}

/// Decode the given string as base64, in an URL-safe manner.
pub fn decode_url(input: &str) -> Result<Vec<u8>, DecodeError> {
    decode(input, base64::URL_SAFE_NO_PAD)
}

/// Decode the given string as base64, with the standaard character set.
pub fn decode_standard(input: &str) -> Result<Vec<u8>, DecodeError> {
    decode(input, base64::STANDARD)
}
