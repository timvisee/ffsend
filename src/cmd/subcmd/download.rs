use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{ArgPassword, ArgUrl, CmdArg};

/// The download command definition.
pub struct CmdDownload;

impl CmdDownload {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        // Build the subcommand
        #[allow(unused_mut)]
        let mut cmd = SubCommand::with_name("download")
            .about("Download files")
            .visible_alias("d")
            .visible_alias("down")
            .arg(ArgUrl::build())
            .arg(ArgPassword::build())
            .arg(
                Arg::with_name("output")
                    .long("output")
                    .short("o")
                    .alias("output-file")
                    .alias("out")
                    .alias("file")
                    .value_name("PATH")
                    .help("Output file or directory"),
            );

        // Optional archive support
        #[cfg(feature = "archive")]
        {
            cmd = cmd.arg(
                Arg::with_name("extract")
                    .long("extract")
                    .short("e")
                    .alias("archive")
                    .alias("arch")
                    .alias("a")
                    .help("Extract an archived file"),
            )
        }

        cmd
    }
}
