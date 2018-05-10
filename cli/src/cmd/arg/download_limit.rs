use clap::{Arg, ArgMatches};
use ffsend_api::action::params::{
    PARAMS_DOWNLOAD_MIN as DOWNLOAD_MIN,
    PARAMS_DOWNLOAD_MAX as DOWNLOAD_MAX,
};

use super::{CmdArg, CmdArgFlag, CmdArgOption};

use util::{ErrorHintsBuilder, quit_error_msg};

/// The download limit argument.
pub struct ArgDownloadLimit { }

impl CmdArg for ArgDownloadLimit {
    fn name() -> &'static str {
        "download-limit"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("download-limit")
            .long("download-limit")
            .short("d")
            .alias("downloads")
            .alias("download")
            .alias("down")
            .alias("dlimit")
            .alias("limit")
            .alias("lim")
            .alias("l")
            .value_name("COUNT")
            .help("The file download limit")
    }
}

impl CmdArgFlag for ArgDownloadLimit { }

impl<'a> CmdArgOption<'a> for ArgDownloadLimit {
    type Value = Option<u8>;

    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value {
        // TODO: do not unwrap, report an error
        Self::value_raw(matches)
            .map(|d| d.parse::<u8>().expect("invalid download limit"))
            .and_then(|d| {
                // Check the download limit bounds
                // TODO: somehow allow to force a different number here
                if d < DOWNLOAD_MIN || d > DOWNLOAD_MAX {
                    quit_error_msg(
                        format!(
                            "invalid download limit, must be between {} and {}",
                            DOWNLOAD_MIN,
                            DOWNLOAD_MAX,
                        ),
                        ErrorHintsBuilder::default()
                            .force(false)
                            .verbose(false)
                            .build()
                            .unwrap(),
                    );
                }

                Some(d)
            })
    }
}
