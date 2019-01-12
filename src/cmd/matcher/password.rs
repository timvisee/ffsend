use clap::ArgMatches;
use ffsend_api::url::Url;
use rpassword::prompt_password_stderr;

use crate::cmd::arg::{ArgGenPassphrase, ArgOwner, ArgPassword, ArgUrl, CmdArgFlag, CmdArgOption};
use crate::cmd::matcher::{MainMatcher, Matcher};
use crate::util::check_empty_password;

/// The password command matcher.
pub struct PasswordMatcher<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> PasswordMatcher<'a> {
    /// Get the file share URL.
    ///
    /// This method parses the URL into an `Url`.
    /// If the given URL is invalid,
    /// the program will quit with an error message.
    pub fn url(&'a self) -> Url {
        ArgUrl::value(self.matches)
    }

    /// Get the owner token.
    pub fn owner(&'a self) -> Option<String> {
        // TODO: just return a string reference here?
        ArgOwner::value(self.matches).map(|token| token.to_owned())
    }

    /// Get the password.
    ///
    /// The password is returned in the following format:
    /// `(password, generated)`
    pub fn password(&'a self) -> (String, bool) {
        // Generate a passphrase if requested
        if ArgGenPassphrase::is_present(self.matches) {
            return (ArgGenPassphrase::gen_passphrase(), true);
        }

        // Get the password, or prompt for it
        let password = match ArgPassword::value(self.matches) {
            Some(password) => password,
            None => {
                // Prompt for the password
                // TODO: don't unwrap/expect
                // TODO: create utility function for this
                prompt_password_stderr("New password: ")
                    .expect("failed to read password from stdin")
            }
        };

        // Create a main matcher
        let matcher_main = MainMatcher::with(self.matches).unwrap();

        // Check for empty passwords
        check_empty_password(&password, &matcher_main);

        (password, false)
    }
}

impl<'a> Matcher<'a> for PasswordMatcher<'a> {
    fn with(matches: &'a ArgMatches) -> Option<Self> {
        matches
            .subcommand_matches("password")
            .map(|matches| PasswordMatcher { matches })
    }
}
