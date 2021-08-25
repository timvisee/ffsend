pub mod debug;
pub mod delete;
pub mod download;
pub mod exists;
pub mod generate;
#[cfg(feature = "history")]
pub mod history;
pub mod info;
pub mod params;
pub mod password;
pub mod upload;
pub mod version;

use ffsend_api::action::version::{Error as VersionError, Version as ApiVersion};
use ffsend_api::api::DesiredVersion;
use ffsend_api::client::Client;
use ffsend_api::url::Url;

use crate::config::API_VERSION_ASSUME;
use crate::util::print_warning;

/// Based on the given desired API version, select a version we can use.
///
/// If the current desired version is set to the `DesiredVersion::Lookup` variant, this method
/// will look up the server API version. It it's `DesiredVersion::Use` it will return and
/// attempt to use the specified version.
fn select_api_version(
    client: &Client,
    host: Url,
    desired: &mut DesiredVersion,
) -> Result<(), VersionError> {
    // Break if already specified
    if let DesiredVersion::Use(_) = desired {
        return Ok(());
    }

    // TODO: only lookup if `DesiredVersion::Assume` after first operation attempt failed

    // Look up the version
    match ApiVersion::new(host).invoke(&client) {
        // Use the probed version
        Ok(v) => *desired = DesiredVersion::Use(v),

        // If unknown, just assume the default version
        Err(VersionError::Unknown) => {
            *desired = DesiredVersion::Use(API_VERSION_ASSUME);
            print_warning(format!(
                "server API version could not be determined, assuming v{}",
                API_VERSION_ASSUME,
            ));
        }

        // Propagate other errors
        Err(e) => return Err(e),
    }

    Ok(())
}
