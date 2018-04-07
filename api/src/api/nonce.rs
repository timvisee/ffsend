use url::Url;
use reqwest::{Client, Response, StatusCode};

use crypto::b64;
use ext::status_code::StatusCodeExt;

/// The name of the header the nonce is delivered in.
const HEADER_NONCE: &'static str = "WWW-Authenticate";

/// Do a new request, and extract the nonce from a header in the given
/// response.
pub fn request_nonce(client: &Client, url: Url)
    -> Result<Vec<u8>, NonceError>
{
    // Make the request
    let response = client.get(url)
        .send()
        .map_err(|_| NonceError::Req)?;

    // Validate the status code
    let status = response.status();
    if !status.is_success() {
        // TODO: should we check here whether a 404 is returned?
        // // Handle expired files
        // if status == FILE_EXPIRED_STATUS {
        //     return Err(Error::Expired);
        // } else {
        return Err(NonceError::ReqStatus(status, status.err_text()).into());
        // }
    }

    // Extract the nonce
    header_nonce(&response)
}

/// Extract the nonce from a header in the given response.
pub fn header_nonce(response: &Response)
    -> Result<Vec<u8>, NonceError>
{
    // Get the authentication nonce
    b64::decode(
        response.headers()
            .get_raw(HEADER_NONCE)
            .ok_or(NonceError::NoNonceHeader)?
            .one()
            .ok_or(NonceError::MalformedNonce)
            .and_then(|line| String::from_utf8(line.to_vec())
                .map_err(|_| NonceError::MalformedNonce)
            )?
            .split_terminator(" ")
            .skip(1)
            .next()
            .ok_or(NonceError::MalformedNonce)?
    ).map_err(|_| NonceError::MalformedNonce.into())
}

#[derive(Fail, Debug)]
pub enum NonceError {
    /// Sending the request to fetch a nonce failed.
    #[fail(display = "Failed to request nonce")]
    Req,

    /// The response for fetching the nonce indicated an error and wasn't
    /// successful.
    #[fail(display = "Bad HTTP response '{}' while requesting nonce", _1)]
    ReqStatus(StatusCode, String),

    /// The nonce header was missing from the request.
    #[fail(display = "Missing nonce in server response")]
    NoNonceHeader,

    /// The received nonce could not be parsed, because it was malformed.
    /// Maybe the server responded with a new format that isn't supported yet
    /// by this client.
    #[fail(display = "Received malformed nonce")]
    MalformedNonce,
}
