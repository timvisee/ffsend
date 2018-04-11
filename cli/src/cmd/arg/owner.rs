use clap::{Arg, ArgMatches};

use super::{CmdArg, CmdArgFlag, CmdArgOption};
use util::prompt_owner_token;

/// The owner argument.
pub struct ArgOwner { }

impl CmdArg for ArgOwner {
    fn name() -> &'static str {
        "owner"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("owner")
            .long("owner")
            .short("o")
            .alias("own")
            .alias("owner-token")
            .alias("token")
            .value_name("TOKEN")
            .min_values(0)
            .max_values(1)
            .help("Specify the file owner token")
    }
}

impl CmdArgFlag for ArgOwner { }

impl<'a> CmdArgOption<'a> for ArgOwner {
    type Value = Option<String>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // The owner token flag must be present
        if !Self::is_present(matches) {
            return None;
        }

        // Get the owner token from the argument if set
        match Self::value_raw(matches) {
            None => {},
            p => return p.map(|p| p.into()),
        }

        // Prompt for the owner token
        Some(prompt_owner_token())
    }
}
