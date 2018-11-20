use std::time::Duration;

/// The timeout for the Send client for generic requests, `None` to disable.
pub const CLIENT_TIMEOUT: Option<Duration> = Some(Duration::from_secs(30));

/// The timeout for the Send client used to transfer (upload/download) files.
/// Make sure this is big enough, or file uploads will be dropped. `None` to disable.
pub const CLIENT_TRANSFER_TIMEOUT: Option<Duration> = Some(Duration::from_secs(24 * 60 * 60));
