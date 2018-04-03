use clap::{Arg, ArgMatches};
use rpassword::prompt_password_stderr;

use super::{CmdArg, CmdArgFlag, CmdArgOption};

/// The password argument.
pub struct ArgPassword { }

impl CmdArg for ArgPassword {
    fn name() -> &'static str {
        "password"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("password")
            .long("password")
            .short("p")
            .alias("pass")
            .value_name("PASSWORD")
            .min_values(0)
            .max_values(1)
            .help("Unlock a password protected file")
    }
}

impl CmdArgFlag for ArgPassword { }

impl<'a> CmdArgOption<'a> for ArgPassword {
    type Value = Option<String>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // The password flag must be present
        if !Self::is_present(matches) {
            return None;
        }

        // Get the password from the argument if set
        match Self::value_raw(matches) {
            None => {},
            p => return p.map(|p| p.into()),
        }

        // Prompt for the password
        // TODO: don't unwrap/expect
        // TODO: create utility function for this
        Some(
            prompt_password_stderr("Password: ")
                .expect("failed to read password from stdin")
        )
    }
}
