extern crate app_dirs2;

use clap::{App, AppSettings, Arg, ArgMatches};
use self::app_dirs2::{AppDataType, AppInfo, get_app_dir};

use super::matcher::{
    DeleteMatcher,
    DownloadMatcher,
    ExistsMatcher,
    InfoMatcher,
    Matcher,
    ParamsMatcher,
    PasswordMatcher,
    UploadMatcher,
};
use super::cmd::{
    CmdDelete,
    CmdDownload,
    CmdExists,
    CmdInfo,
    CmdParams,
    CmdPassword,
    CmdUpload,
};

lazy_static! {
    /// The default history file
    static ref DEFAULT_HISTORY_FILE: String =
        get_app_dir(
            AppDataType::UserCache,
            &AppInfo {
                name: crate_name!(),
                author: crate_name!(),
            },
            "history.toml",
        )
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned();
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
            .about(crate_description!())
            .global_setting(AppSettings::GlobalVersion) .global_setting(AppSettings::VersionlessSubcommands)
            // TODO: enable below command when it doesn't break `p` anymore.
            // .global_setting(AppSettings::InferSubcommands)
            .arg(Arg::with_name("force")
                .long("force")
                .short("f")
                .global(true)
                .help("Force the action, ignore warnings"))
            .arg(Arg::with_name("no-interact")
                .long("no-interact")
                .short("I")
                .alias("no-interactive")
                .global(true)
                .help("Not interactive, do not prompt"))
            .arg(Arg::with_name("yes")
                .long("yes")
                .short("y")
                .alias("assume-yes")
                .global(true)
                .help("Assume yes for prompts"))
            .arg(Arg::with_name("history")
                .long("history")
                .short("H")
                .value_name("FILE")
                .global(true)
                .help("History file to use")
                .default_value(&DEFAULT_HISTORY_FILE))
            .arg(Arg::with_name("incognito")
                .long("incognito")
                .short("i")
                .alias("incog")
                .alias("private")
                .alias("priv")
                .global(true)
                .help("Don't update local history for actions"))
            .subcommand(CmdDelete::build())
            .subcommand(CmdDownload::build().display_order(2))
            .subcommand(CmdExists::build())
            .subcommand(CmdInfo::build())
            .subcommand(CmdParams::build())
            .subcommand(CmdPassword::build())
            .subcommand(CmdUpload::build().display_order(1));

        // Disable color usage if compiled without color support
        #[cfg(feature = "no-color")]
        let app = app.global_setting(AppSettings::ColorNever);

        app
    }

    /// Parse CLI arguments.
    pub fn parse() -> Handler<'a> {
        // Build the application CLI definition, get the matches
        Handler {
            matches: Handler::build().get_matches(),
        }
    }

    /// Get the raw matches.
    pub fn matches(&'a self) -> &'a ArgMatches {
        &self.matches
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
}
