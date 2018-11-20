/// The timeout for the Send client for generic requests, `0` to disable.
pub const CLIENT_TIMEOUT: u64 = 30;

/// The timeout for the Send client used to transfer (upload/download) files.
/// Make sure this is big enough, or file uploads will be dropped. `0` to disable.
pub const CLIENT_TRANSFER_TIMEOUT: u64 = 24 * 60 * 60;
