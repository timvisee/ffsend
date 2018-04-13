use reqwest::{Response, StatusCode};

use config::{HTTP_STATUS_EXPIRED, HTTP_STATUS_UNAUTHORIZED};
use ext::status_code::StatusCodeExt;

/// Ensure the given response is successful. IF it isn
pub fn ensure_success(response: &Response) -> Result<(), ResponseError> {
    // Get the status
    let status = response.status();

    // Stop if succesful
    if status.is_success() {
        return Ok(());
    }

    // Handle the expired file error
    if status == HTTP_STATUS_EXPIRED {
        return Err(ResponseError::Expired);
    }

    // Handle the authentication issue error
    if status == HTTP_STATUS_UNAUTHORIZED {
        return Err(ResponseError::Unauthorized);
    }

    // Return the other error
    Err(ResponseError::Other(status, status.err_text()))
}

#[derive(Fail, Debug)]
pub enum ResponseError {
    /// This request lead to an expired file, or a file that never existed.
    #[fail(display = "This file has expired or did never exist")]
    Expired,

    /// We were unauthorized to make this request.
    /// This is usually because of an incorrect password.
    #[fail(display = "Unauthorized, are the credentials correct?")]
    Unauthorized,

    /// Some undefined error occurred with this response.
    #[fail(display = "Bad HTTP response: {}", _1)]
    Other(StatusCode, String),
}
