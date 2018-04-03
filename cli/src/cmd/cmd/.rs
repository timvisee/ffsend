use ffsend_api::url::{ParseError, Url};

use clap::{App, Arg, ArgMatches, SubCommand};
use rpassword::prompt_password_stderr;

use util::quit_error_msg;

/// The password command.
pub struct CmdPassword<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CmdPassword<'a> {
    /// Build the sub command definition.
    pub fn build<'y, 'z>() -> App<'y, 'z> {
        // Build the subcommand
        let cmd = SubCommand::with_name("password")
            .about("Change the password of a shared file.")
            .visible_alias("p")
            .visible_alias("pass")
            .arg(Arg::with_name("URL")
                .help("The share URL")
                .required(true)
                .multiple(false))
            .arg(Arg::with_name("password")
                .long("password")
                .short("p")
                .alias("pass")
                .value_name("PASSWORD")
                .help("Specify a password, do not prompt"))
            .arg(Arg::with_name("owner")
                .long("owner")
                .short("o")
                .alias("own")
                .alias("owner-token")
                .alias("token")
                .value_name("TOKEN")
                .help("File owner token"));

        cmd
    }

    /// Parse CLI arguments, from the given parent command matches.
    pub fn parse(parent: &'a ArgMatches<'a>) -> Option<CmdPassword<'a>> {
        parent.subcommand_matches("password")
            .map(|matches| CmdPassword { matches })
    }

    /// Get the file share URL.
    ///
    /// This method parses the URL into an `Url`.
    /// If the given URL is invalid,
    /// the program will quit with an error message.
    pub fn url(&'a self) -> Url {
        // Get the host
        let url = self.matches.value_of("URL")
            .expect("missing URL");

        // Parse the URL
        // TODO: improve these error messages
        match Url::parse(url) {
            Ok(url) => url,
            Err(ParseError::EmptyHost) =>
                quit_error_msg("Emtpy host given"),
            Err(ParseError::InvalidPort) =>
                quit_error_msg("Invalid host port"),
            Err(ParseError::InvalidIpv4Address) =>
                quit_error_msg("Invalid IPv4 address in host"),
            Err(ParseError::InvalidIpv6Address) =>
                quit_error_msg("Invalid IPv6 address in host"),
            Err(ParseError::InvalidDomainCharacter) =>
                quit_error_msg("Host domains contains an invalid character"),
            Err(ParseError::RelativeUrlWithoutBase) =>
                quit_error_msg("Host domain doesn't contain a host"),
            _ => quit_error_msg("The given host is invalid"),
        }
    }

    /// Get the owner token.
    pub fn owner(&'a self) -> Option<String> {
        // TODO: validate the owner token if set
        self.matches.value_of("owner")
            .map(|token| token.to_owned())
    }

    /// Get the password.
    pub fn password(&'a self) -> String {
        // Get the password from the arguments
        if let Some(password) = self.matches.value_of("password") {
            return password.into();
        }

        // Prompt for the password
        // TODO: don't unwrap/expect
        // TODO: create utility function for this
        prompt_password_stderr("New password: ")
            .expect("failed to read password from stdin")
    }
}
