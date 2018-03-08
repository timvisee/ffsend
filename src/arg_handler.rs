extern crate clap;

use self::clap::{Arg, ArgMatches, App};

use app::*;

/// CLI argument handler.
pub struct ArgHandler<'a> {
    matches: ArgMatches<'a>,
}

impl<'a: 'b, 'b> ArgHandler<'a> {
    /// Parse CLI arguments.
    pub fn parse() -> ArgHandler<'a> {
        // Handle/parse arguments
        let matches = App::new(APP_NAME)
            .version(APP_VERSION)
            .author(APP_AUTHOR)
            .about(APP_ABOUT)
            .arg(Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("PATH")
                .help("The file to upload")
                .required(true)
                .multiple(false))
            .get_matches();

        // Instantiate
        ArgHandler {
            matches,
        }
    }

    /// Get the selected file to upload.
    pub fn file(&'a self) -> &'a str {
        self.matches.value_of("file")
            .expect("please specify a file to upload")
    }
}

