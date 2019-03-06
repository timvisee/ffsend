use clap::{Arg, ArgMatches};

use super::{CmdArg, CmdArgFlag, CmdArgOption};
use crate::cmd::matcher::{MainMatcher, Matcher};
use crate::util::{check_empty_password, prompt_password};

/// The password argument.
pub struct ArgPassword {}

impl CmdArg for ArgPassword {
    fn name() -> &'static str {
        "password"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("password")
            .long("password")
            .short("p")
            .value_name("PASSWORD")
            .min_values(0)
            .max_values(1)
            .help("Unlock a password protected file")
    }
}

impl CmdArgFlag for ArgPassword {}

impl<'a> CmdArgOption<'a> for ArgPassword {
    type Value = Option<String>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // The password flag must be present
        if !Self::is_present(matches) {
            return None;
        }

        // Create a main matcher
        let matcher_main = MainMatcher::with(matches).unwrap();

        // Get the password argument value, or prompt
        let password = match Self::value_raw(matches) {
            Some(password) => password.into(),
            None => prompt_password(&matcher_main, false).unwrap(),
        };

        // Check for empty passwords
        check_empty_password(&password, &matcher_main);

        Some(password)
    }
}
