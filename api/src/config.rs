use reqwest::StatusCode;

/// The Send host to use by default.
pub const SEND_DEFAULT_HOST: &'static str = "https://send.firefox.com/";

/// The default time after which uploaded files expire after, in seconds.
pub const SEND_DEFAULT_EXPIRE_TIME: i64 = 24 * 60 * 60;

/// The HTTP status code that is returned for expired or non existant files.
pub const HTTP_STATUS_EXPIRED: StatusCode = StatusCode::NotFound;

/// The HTTP status code that is returned on authentication failure.
pub const HTTP_STATUS_UNAUTHORIZED: StatusCode = StatusCode::Unauthorized;

/// The recommended maximum upload size in bytes.
pub const UPLOAD_SIZE_MAX_RECOMMENDED: u64 = 1024 * 1024 * 1024 * 1;

/// The maximum upload size in bytes.
pub const UPLOAD_SIZE_MAX: u64 = 1024 * 1024 * 1024 * 2;
