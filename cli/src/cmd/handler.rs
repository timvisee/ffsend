use super::clap::{App, AppSettings, Arg, ArgMatches};

use app::*;

use super::cmd_delete::CmdDelete;
use super::cmd_download::CmdDownload;
use super::cmd_info::CmdInfo;
use super::cmd_params::CmdParams;
use super::cmd_password::CmdPassword;
use super::cmd_upload::CmdUpload;

/// CLI argument handler.
pub struct Handler<'a> {
    /// The CLI matches.
    matches: ArgMatches<'a>,
}

impl<'a: 'b, 'b> Handler<'a> {
    /// Build the application CLI definition.
    pub fn build() -> App<'a, 'b> {
        App::new(APP_NAME)
            .version(APP_VERSION)
            .author(APP_AUTHOR)
            .about(APP_ABOUT)
            .global_setting(AppSettings::GlobalVersion)
            .global_setting(AppSettings::VersionlessSubcommands)
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
                .visible_alias("assume-yes")
                .global(true)
                .help("Assume yes for prompts"))
            .subcommand(CmdDelete::build())
            .subcommand(CmdDownload::build().display_order(2))
            .subcommand(CmdInfo::build())
            .subcommand(CmdParams::build())
            .subcommand(CmdPassword::build())
            .subcommand(CmdUpload::build().display_order(1))
    }

    /// Parse CLI arguments.
    pub fn parse() -> Handler<'a> {
        // Build the application CLI definition, get the matches
        Handler {
            matches: Handler::build().get_matches(),
        }
    }

    /// Get the delete sub command, if matched.
    pub fn delete(&'a self) -> Option<CmdDelete<'a>> {
        CmdDelete::parse(&self.matches)
    }

    /// Get the download sub command, if matched.
    pub fn download(&'a self) -> Option<CmdDownload<'a>> {
        CmdDownload::parse(&self.matches)
    }

    /// Get the info sub command, if matched.
    pub fn info(&'a self) -> Option<CmdInfo<'a>> {
        CmdInfo::parse(&self.matches)
    }

    /// Get the parameters sub command, if matched.
    pub fn params(&'a self) -> Option<CmdParams<'a>> {
        CmdParams::parse(&self.matches)
    }

    /// Get the password sub command, if matched.
    pub fn password(&'a self) -> Option<CmdPassword<'a>> {
        CmdPassword::parse(&self.matches)
    }

    /// Get the upload sub command, if matched.
    pub fn upload(&'a self) -> Option<CmdUpload<'a>> {
        CmdUpload::parse(&self.matches)
    }
}
