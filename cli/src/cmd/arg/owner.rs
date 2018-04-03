use clap::{Arg, ArgMatches};

use super::{CmdArg, CmdArgOption};

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
            .help("Specify the file owner token")
    }
}

impl<'a> CmdArgOption<'a> for ArgOwner {
    type Value = Option<&'a str>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        Self::value_raw(matches)
    }
}
