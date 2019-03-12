use ffsend_api::api::{DesiredVersion, Version};

/// The timeout for the Send client for generic requests, `0` to disable.
pub const CLIENT_TIMEOUT: u64 = 30;

/// The timeout for the Send client used to transfer (upload/download) files.
/// Make sure this is big enough, or file uploads will be dropped. `0` to disable.
pub const CLIENT_TRANSFER_TIMEOUT: u64 = 24 * 60 * 60;

/// The default desired version to select for the server API.
pub const API_VERSION_DESIRED_DEFAULT: DesiredVersion = DesiredVersion::Assume(API_VERSION_ASSUME);

/// The default server API version to assume when it could not be determined.
#[cfg(feature = "send3")]
pub const API_VERSION_ASSUME: Version = Version::V3;
#[cfg(not(feature = "send3"))]
pub const API_VERSION_ASSUME: Version = Version::V2;
