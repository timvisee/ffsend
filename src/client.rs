use std::time::Duration;

use ffsend_api::reqwest::{Client, ClientBuilder};

use config::{CLIENT_TIMEOUT, CLIENT_TRANSFER_TIMEOUT};

/// Create the default client, which is used for generic Send requests.
///
/// Note: use `create_transfer_client()` instead for clients that upload/download.
pub fn create_client() -> Client {
    create_custom_client(CLIENT_TIMEOUT)
}

/// Create the default client, which is used for generic Send requests.
///
/// Note: use `create_transfer_client()` instead for clients that upload/download.
pub fn create_transfer_client() -> Client {
    create_custom_client(CLIENT_TRANSFER_TIMEOUT)
}

/// Create the Send client with a custom timeout.
fn create_custom_client(timeout: Option<Duration>) -> Client {
    ClientBuilder::new()
        .timeout(timeout)
        .build()
        .expect("failed to build custom reqwest client")
}
