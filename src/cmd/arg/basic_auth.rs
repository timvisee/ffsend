use clap::{Arg, ArgMatches};

use super::{CmdArg, CmdArgOption};

/// The basicauth argument.
pub struct ArgBasicAuth {}

impl CmdArg for ArgBasicAuth {
    fn name() -> &'static str {
        "basic-auth"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("basic-auth")
            .long("basic-auth")
            .alias("basic-authentication")
            .alias("http-basic-authentication")
            .alias("http-basic-auth")
            .value_name("USER:PASSWORD")
            .env("FFSEND_BASIC_AUTH")
            .hide_env_values(true)
            .global(true)
            .help("Protected proxy HTTP basic authentication credentials (not FxA)")
    }
}

impl<'a> CmdArgOption<'a> for ArgBasicAuth {
    type Value = Option<(String, Option<String>)>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // Get the authentication credentials
        let raw = match Self::value_raw(matches) {
            Some(raw) => raw,
            None => return None,
        };

        // Split the properties
        let mut iter = raw.splitn(2, ':');
        Some((
            iter.next().unwrap_or("").to_owned(),
            iter.next().map(|pass| pass.to_owned()),
        ))
    }
}
