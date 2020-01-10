use clap::{App, Arg, SubCommand};

use crate::cmd::arg::{
    ArgDownloadLimit, ArgExpiryTime, ArgGenPassphrase, ArgHost, ArgPassword, CmdArg,
};

/// The upload command definition.
pub struct CmdUpload;

impl CmdUpload {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        // Build the subcommand
        #[allow(unused_mut)]
        let mut cmd = SubCommand::with_name("upload")
            .about("Upload files")
            .visible_alias("u")
            .visible_alias("up")
            .arg(
                Arg::with_name("FILE")
                    .help("The file(s) to upload")
                    .required(true)
                    .multiple(true),
            )
            .arg(ArgPassword::build().help("Protect the file with a password"))
            .arg(ArgGenPassphrase::build())
            .arg(ArgDownloadLimit::build())
            .arg(ArgExpiryTime::build())
            .arg(ArgHost::build())
            .arg(
                Arg::with_name("name")
                    .long("name")
                    .short("n")
                    .alias("file")
                    .alias("f")
                    .value_name("NAME")
                    .help("Rename the file being uploaded"),
            )
            .arg(
                Arg::with_name("open")
                    .long("open")
                    .short("o")
                    .help("Open the share link in your browser"),
            )
            .arg(
                Arg::with_name("delete")
                    .long("delete")
                    .alias("rm")
                    .short("D")
                    .help("Delete local file after upload"),
            );

        // Optional archive support
        #[cfg(feature = "archive")]
        {
            cmd = cmd.arg(
                Arg::with_name("archive")
                    .long("archive")
                    .short("a")
                    .alias("arch")
                    .help("Archive the upload in a single file"),
            )
        }

        // Optional clipboard support
        #[cfg(feature = "clipboard")]
        {
            cmd = cmd
                .arg(
                    Arg::with_name("copy")
                        .long("copy")
                        .short("c")
                        .help("Copy the share link to your clipboard")
                        .conflicts_with("copy-cmd"),
                )
                .arg(
                    Arg::with_name("copy-cmd")
                        .long("copy-cmd")
                        .alias("copy-command")
                        .short("C")
                        .help("Copy the ffsend download command to your clipboard")
                        .conflicts_with("copy"),
                );
        }

        // Optional url shortening support
        #[cfg(feature = "urlshorten")]
        {
            cmd = cmd.arg(
                Arg::with_name("shorten")
                    .long("shorten")
                    .alias("short")
                    .alias("url-shorten")
                    .short("S")
                    .help("Shorten share URLs with a public service"),
            )
        }

        // Optional qrcode support
        #[cfg(feature = "qrcode")]
        {
            cmd = cmd.arg(
                Arg::with_name("qrcode")
                    .long("qrcode")
                    .alias("qr")
                    .short("Q")
                    .help("Print a QR code for the share URL"),
            )
        }

        cmd
    }
}
