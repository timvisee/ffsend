use std::path::PathBuf;

use ffsend_api::url::{ParseError, Url};

use rpassword::prompt_password_stderr;
use super::clap::{App, Arg, ArgMatches, SubCommand};

use util::quit_error_msg;

/// The download command.
pub struct CmdDownload<'a> {
    matches: &'a ArgMatches<'a>,
}

impl<'a: 'b, 'b> CmdDownload<'a> {
    /// Build the sub command definition.
    pub fn build<'y, 'z>() -> App<'y, 'z> {
        // Build the subcommand
        let cmd = SubCommand::with_name("download")
            .about("Download files.")
            .visible_alias("d")
            .visible_alias("down")
            .arg(Arg::with_name("URL")
                .help("The share URL")
                .required(true)
                .multiple(false))
            .arg(Arg::with_name("output")
                .long("output")
                .short("o")
                .alias("output-file")
                .alias("out")
                .alias("file")
                .value_name("PATH")
                .help("The output file or directory"))
            .arg(Arg::with_name("password")
                .long("password")
                .short("p")
                .alias("pass")
                .value_name("PASSWORD")
                .min_values(0)
                .max_values(1)
                .help("Unlock a password protected file"));

        cmd
    }

    /// Parse CLI arguments, from the given parent command matches.
    pub fn parse(parent: &'a ArgMatches<'a>) -> Option<CmdDownload<'a>> {
        parent.subcommand_matches("download")
            .map(|matches| CmdDownload { matches })
    }

    /// Get the file share URL, to download the file from.
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

    /// The target file or directory to download the file to.
    /// If a directory is given, the file name of the original uploaded file
    /// will be used.
    pub fn output(&'a self) -> PathBuf {
        self.matches.value_of("output")
            .map(|path| PathBuf::from(path))
            .unwrap_or(PathBuf::from("./"))
    }

    /// Get the password.
    /// `None` is returned if no password was specified.
    pub fn password(&'a self) -> Option<String> {
        // Return none if the property was not set
        if !self.matches.is_present("password") {
            return None;
        }

        // Get the password from the arguments
        if let Some(password) = self.matches.value_of("password") {
            return Some(password.into());
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
