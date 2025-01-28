#[cfg(feature = "infer-command")]
use std::ffi::OsString;

use clap::{App, AppSettings, Arg, ArgMatches};

use super::arg::{ArgApi, ArgBasicAuth, CmdArg};
#[cfg(feature = "history")]
use super::matcher::HistoryMatcher;
use super::matcher::{
    DebugMatcher, DeleteMatcher, DownloadMatcher, ExistsMatcher, GenerateMatcher, InfoMatcher,
    Matcher, ParamsMatcher, PasswordMatcher, UploadMatcher, VersionMatcher,
};
#[cfg(feature = "history")]
use super::subcmd::CmdHistory;
use super::subcmd::{
    CmdDebug, CmdDelete, CmdDownload, CmdExists, CmdGenerate, CmdInfo, CmdParams, CmdPassword,
    CmdUpload, CmdVersion,
};
#[cfg(feature = "infer-command")]
use crate::config::INFER_COMMANDS;
use crate::config::{CLIENT_TIMEOUT, CLIENT_TRANSFER_TIMEOUT};
#[cfg(feature = "history")]
use crate::util::app_history_file_path_string;
#[cfg(feature = "infer-command")]
use crate::util::bin_name;
use crate::util::parse_duration;

#[cfg(feature = "history")]
lazy_static! {
    /// The default history file
    static ref DEFAULT_HISTORY_FILE: String = app_history_file_path_string();
}

lazy_static! {
    /// The default client timeout in seconds as a string
    static ref DEFAULT_TIMEOUT: String = format!("{}", CLIENT_TIMEOUT);

    /// The default client transfer timeout in seconds as a string
    static ref DEFAULT_TRANSFER_TIMEOUT: String = format!("{}", CLIENT_TRANSFER_TIMEOUT);

    /// The about notice in command output.
    static ref APP_ABOUT: String = format!(
        "{}\n\n\
            The default public Send host is provided by Tim Visee, @timvisee.\n\
            Please consider to donate and help keep it running: https://vis.ee/donate\
            ",
        crate_description!(),
    );
}

/// CLI argument handler.
pub struct Handler<'a> {
    /// The CLI matches.
    matches: ArgMatches<'a>,
}

impl<'a: 'b, 'b> Handler<'a> {
    /// Build the application CLI definition.
    pub fn build() -> App<'a, 'b> {
        // Build the CLI application definition
        let app = App::new(crate_name!())
            .version(crate_version!())
            .author(crate_authors!())
            .about(APP_ABOUT.as_str())
            .after_help("This application is not affiliated with Firefox or Mozilla.")
            .global_setting(AppSettings::GlobalVersion)
            .global_setting(AppSettings::VersionlessSubcommands)
            // TODO: enable below command when it doesn't break `p` anymore.
            // .global_setting(AppSettings::InferSubcommands)
            .arg(
                Arg::with_name("force")
                    .long("force")
                    .short("f")
                    .global(true)
                    .help("Force the action, ignore warnings"),
            )
            .arg(
                Arg::with_name("no-interact")
                    .long("no-interact")
                    .short("I")
                    .alias("no-interactive")
                    .alias("non-interactive")
                    .global(true)
                    .help("Not interactive, do not prompt"),
            )
            .arg(
                Arg::with_name("yes")
                    .long("yes")
                    .short("y")
                    .alias("assume-yes")
                    .global(true)
                    .help("Assume yes for prompts"),
            )
            .arg(
                Arg::with_name("timeout")
                    .long("timeout")
                    .short("t")
                    .alias("time")
                    .global(true)
                    .value_name("SECONDS")
                    .help("Request timeout (0 to disable)")
                    .default_value(&DEFAULT_TIMEOUT)
                    .hide_default_value(true)
                    .env("FFSEND_TIMEOUT")
                    .hide_env_values(true)
                    .validator(|arg| {
                        parse_duration(&arg).map(drop).map_err(|_| {
                            String::from(
                            "Timeout time must be a positive number of seconds, or 0 to disable."
                        )
                        })
                    }),
            )
            .arg(
                Arg::with_name("transfer-timeout")
                    .long("transfer-timeout")
                    .short("T")
                    .alias("trans-time")
                    .alias("trans-timeout")
                    .alias("transfer-time")
                    .alias("time-trans")
                    .alias("timeout-trans")
                    .alias("time-transfer")
                    .global(true)
                    .value_name("SECONDS")
                    .help("Transfer timeout (0 to disable)")
                    .default_value(&DEFAULT_TRANSFER_TIMEOUT)
                    .hide_default_value(true)
                    .env("FFSEND_TRANSFER_TIMEOUT")
                    .hide_env_values(true)
                    .validator(|arg| {
                        parse_duration(&arg).map(drop).map_err(|_| {
                            String::from(
                            "Timeout time must be a positive number of seconds, or 0 to disable."
                        )
                        })
                    }),
            )
            .arg(
                Arg::with_name("quiet")
                    .long("quiet")
                    .short("q")
                    .global(true)
                    .help("Produce output suitable for logging and automation"),
            )
            .arg(
                Arg::with_name("verbose")
                    .long("verbose")
                    .short("v")
                    .multiple(true)
                    .global(true)
                    .help("Enable verbose information and logging"),
            )
            .arg(ArgApi::build())
            .arg(ArgBasicAuth::build())
            .subcommand(CmdDebug::build())
            .subcommand(CmdDelete::build())
            .subcommand(CmdDownload::build().display_order(2))
            .subcommand(CmdExists::build())
            .subcommand(CmdGenerate::build())
            .subcommand(CmdInfo::build())
            .subcommand(CmdParams::build())
            .subcommand(CmdPassword::build())
            .subcommand(CmdUpload::build().display_order(1))
            .subcommand(CmdVersion::build());

        // With history support, a flag for the history file and incognito mode
        #[cfg(feature = "history")]
        let app = app
            .arg(
                Arg::with_name("history")
                    .long("history")
                    .short("H")
                    .value_name("FILE")
                    .global(true)
                    .help("Use the specified history file")
                    .default_value(&DEFAULT_HISTORY_FILE)
                    .hide_default_value(true)
                    .env("FFSEND_HISTORY")
                    .hide_env_values(true),
            )
            .arg(
                Arg::with_name("incognito")
                    .long("incognito")
                    .short("i")
                    .alias("incog")
                    .alias("private")
                    .alias("priv")
                    .global(true)
                    .help("Don't update local history for actions"),
            )
            .subcommand(CmdHistory::build());

        // Disable color usage if compiled without color support
        #[cfg(feature = "no-color")]
        let app = app.global_setting(AppSettings::ColorNever);

        app
    }

    /// Parse CLI arguments.
    pub fn parse() -> Handler<'a> {
        // Obtain the program arguments
        #[allow(unused_mut)]
        let mut args: Vec<_> = ::std::env::args_os().collect();

        // Infer subcommand based on binary name
        #[cfg(feature = "infer-command")]
        Self::infer_subcommand(&mut args);

        // Build the application CLI definition, get the matches
        Handler {
            matches: Handler::build().get_matches_from(args),
        }
    }

    /// Infer subcommand when the binary has a predefined name,
    /// modifying the given program arguments.
    ///
    /// If no subcommand could be inferred, the `args` list leaves unchanged.
    /// See `crate::config::INFER_COMMANDS` for a list of commands.
    ///
    /// When the `ffsend` binary is called with such a name, the corresponding subcommand is
    /// automatically inserted as argument. This also works when calling binaries through symbolic
    /// or hard links.
    #[cfg(feature = "infer-command")]
    fn infer_subcommand(args: &mut Vec<OsString>) {
        // Get the name of the called binary
        let name = bin_name();

        // Infer subcommands
        for (bin, subcmd) in INFER_COMMANDS.iter() {
            if &name == bin {
                args.insert(1, subcmd.into());
                break;
            }
        }
    }

    /// Get the raw matches.
    pub fn matches(&'a self) -> &'a ArgMatches {
        &self.matches
    }

    /// Get the debug sub command, if matched.
    pub fn debug(&'a self) -> Option<DebugMatcher> {
        DebugMatcher::with(&self.matches)
    }

    /// Get the delete sub command, if matched.
    pub fn delete(&'a self) -> Option<DeleteMatcher> {
        DeleteMatcher::with(&self.matches)
    }

    /// Get the download sub command, if matched.
    pub fn download(&'a self) -> Option<DownloadMatcher> {
        DownloadMatcher::with(&self.matches)
    }

    /// Get the exists sub command, if matched.
    pub fn exists(&'a self) -> Option<ExistsMatcher> {
        ExistsMatcher::with(&self.matches)
    }

    /// Get the generate sub command, if matched.
    pub fn generate(&'a self) -> Option<GenerateMatcher> {
        GenerateMatcher::with(&self.matches)
    }

    /// Get the history sub command, if matched.
    #[cfg(feature = "history")]
    pub fn history(&'a self) -> Option<HistoryMatcher> {
        HistoryMatcher::with(&self.matches)
    }

    /// Get the info matcher, if that subcommand is entered.
    pub fn info(&'a self) -> Option<InfoMatcher> {
        InfoMatcher::with(&self.matches)
    }

    /// Get the parameters sub command, if matched.
    pub fn params(&'a self) -> Option<ParamsMatcher> {
        ParamsMatcher::with(&self.matches)
    }

    /// Get the password sub command, if matched.
    pub fn password(&'a self) -> Option<PasswordMatcher> {
        PasswordMatcher::with(&self.matches)
    }

    /// Get the upload sub command, if matched.
    pub fn upload(&'a self) -> Option<UploadMatcher> {
        UploadMatcher::with(&self.matches)
    }

    /// Get the version sub command, if matched.
    pub fn version(&'a self) -> Option<VersionMatcher> {
        VersionMatcher::with(&self.matches)
    }
}
