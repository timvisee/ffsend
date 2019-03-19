use std::time::Duration;

use ffsend_api::client::{ClientConfig, ClientConfigBuilder};

use crate::cmd::matcher::MainMatcher;

/// Create a client configuration for ffsend actions.
///
/// A client configuration allows you to build a client, which must be passed to ffsend API
/// actions.
// TODO: properly handle errors, do not unwrap
pub fn create_config(matcher_main: &MainMatcher) -> ClientConfig {
    // TODO: configure HTTP authentication properties
    ClientConfigBuilder::default()
        .timeout(to_duration(matcher_main.timeout()))
        .transfer_timeout(to_duration(matcher_main.transfer_timeout()))
        .basic_auth(matcher_main.basic_auth())
        .build()
        .expect("failed to create network client configuration")
}

/// Convert the given number of seconds into an optional duration, used for clients.
pub fn to_duration(secs: u64) -> Option<Duration> {
    if secs > 0 {
        Some(Duration::from_secs(secs))
    } else {
        None
    }
}
