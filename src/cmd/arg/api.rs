use clap::{Arg, ArgMatches};
use ffsend_api::api::{DesiredVersion, Version};

use super::{CmdArg, CmdArgOption};
use crate::config::API_VERSION_DESIRED_DEFAULT;

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
        if version.trim() == "-" {
            return DesiredVersion::Lookup;
        }

        // Parse the given API version
        match Version::parse(version) {
            Ok(version) => DesiredVersion::Use(version),
            Err(_) => {
                panic!("failed to determine given server API version, version unknown");

                // TODO: properly quit with an application error instead
                // quit_error(
                //     err.context("failed to determine given server API version, version unknown"),
                //     ErrorHints::default(),
                // )
            },
        }
    }
}
