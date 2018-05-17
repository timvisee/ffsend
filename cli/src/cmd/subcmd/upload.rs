use clap::{App, Arg, SubCommand};
use ffsend_api::action::params::{
    PARAMS_DEFAULT_DOWNLOAD_STR as DOWNLOAD_DEFAULT,
};

use cmd::arg::{ArgDownloadLimit, ArgHost, ArgPassword, CmdArg};

/// The uplaod command definition.
pub struct CmdUpload;

impl CmdUpload {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        // Build the subcommand
        #[allow(unused_mut)]
        let mut cmd = SubCommand::with_name("upload")
            .about("Upload files")
            .visible_alias("u")
            .visible_alias("up")
            .arg(Arg::with_name("FILE")
                .help("The file to upload")
                .required(true)
                .multiple(false))
            .arg(ArgPassword::build()
                 .help("Protect the file with a password"))
            .arg(ArgDownloadLimit::build()
                 .default_value(DOWNLOAD_DEFAULT))
            .arg(ArgHost::build())
            .arg(Arg::with_name("name")
                .long("name")
                .short("n")
                .alias("file")
                .alias("f")
                .value_name("NAME")
                .help("Rename the file being uploaded"))
            .arg(Arg::with_name("open")
                .long("open")
                .short("o")
                .env("FFSEND_OPEN")
                .help("Open the share link in your browser"));

        // Optional archive support
        #[cfg(feature = "archive")] {
            cmd = cmd.arg(Arg::with_name("archive")
                .long("archive")
                .short("a")
                .alias("arch")
                .env("FFSEND_ARCHIVE")
                .help("Package the file as an archive"))
        }

        // Optional clipboard support
        #[cfg(feature = "clipboard")] {
            cmd = cmd.arg(Arg::with_name("copy")
                .long("copy")
                .short("c")
                .env("FFSEND_COPY")
                .help("Copy the share link to your clipboard"));
        } 
        cmd
    }
}
