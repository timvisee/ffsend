use clap::{Arg, ArgMatches};
use ffsend_api::api::{DesiredVersion, Version};

use super::{CmdArg, CmdArgOption};
use crate::config::API_VERSION_DESIRED_DEFAULT;
use crate::util::{quit_error_msg, ErrorHints};

/// The api argument.
pub struct ArgApi {}

impl CmdArg for ArgApi {
    fn name() -> &'static str {
        "api"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("api")
            .long("api")
            .short("A")
            .value_name("VERSION")
            .env("FFSEND_API")
            .hide_env_values(true)
            .global(true)
            .help("Server API version to use, '-' to lookup")
            .long_help(
                "Server API version to use, one of:\n\
                 2, 3: Send API versions\n\
                 auto, -: probe server to determine\
                 ",
            )
    }
}

impl<'a> CmdArgOption<'a> for ArgApi {
    type Value = DesiredVersion;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // Get the version string
        let version = match Self::value_raw(matches) {
            Some(version) => version,
            None => return API_VERSION_DESIRED_DEFAULT,
        };

        // Parse the lookup version string
        if is_auto(version) {
            return DesiredVersion::Lookup;
        }

        // Parse the given API version
        match Version::parse(version) {
            Ok(version) => DesiredVersion::Use(version),
            Err(_) => quit_error_msg(
                "failed to determine given server API version, version unknown",
                ErrorHints::default(),
            ),
        }
    }
}

/// Check whether the given API version argument means we've to probe the server for the proper
/// version.
fn is_auto(arg: &str) -> bool {
    let arg = arg.trim().to_lowercase();
    arg == "a" || arg == "auto" || arg == "-"
}
