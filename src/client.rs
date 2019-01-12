use std::time::Duration;

use ffsend_api::reqwest::{Client, ClientBuilder};

use crate::cmd::matcher::MainMatcher;

/// Create the default client, which is used for generic Send requests.
///
/// Note: use `create_transfer_client()` instead for clients that upload/download.
pub fn create_client(matcher_main: &MainMatcher) -> Client {
    create_custom_client(to_duration(matcher_main.timeout()))
}

/// Create the default client, which is used for generic Send requests.
///
/// Note: use `create_transfer_client()` instead for clients that upload/download.
pub fn create_transfer_client(matcher_main: &MainMatcher) -> Client {
    create_custom_client(to_duration(matcher_main.transfer_timeout()))
}

/// Create the Send client with a custom timeout.
fn create_custom_client(timeout: Option<Duration>) -> Client {
    ClientBuilder::new()
        .timeout(timeout)
        .build()
        .expect("failed to build custom reqwest client")
}

/// Convert the given number of seconds into an optional duration, used for clients.
pub fn to_duration(secs: u64) -> Option<Duration> {
    if secs > 0 {
        Some(Duration::from_secs(secs))
    } else {
        None
    }
}
