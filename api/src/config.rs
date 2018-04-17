use reqwest::StatusCode;

/// The default Send host to use.
pub const SEND_DEFAULT_HOST: &'static str = "https://send.firefox.com/";

/// The HTTP status code that is returned for expired or non existant files.
pub const HTTP_STATUS_EXPIRED: StatusCode = StatusCode::NotFound;

/// The HTTP status code that is returned on authentication failure.
pub const HTTP_STATUS_UNAUTHORIZED: StatusCode = StatusCode::Unauthorized;

/// The recommended maximum upload size in bytes.
pub const UPLOAD_SIZE_MAX_RECOMMENDED: u64 = 1024 * 1024 * 1024 * 1;

/// The maximum upload size in bytes.
pub const UPLOAD_SIZE_MAX: u64 = 1024 * 1024 * 1024 * 2;
