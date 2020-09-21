#[macro_use]
extern crate clap;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "history")]
#[macro_use]
extern crate serde_derive;

mod action;
#[cfg(feature = "archive")]
mod archive;
mod client;
mod cmd;
mod config;
mod error;
#[cfg(feature = "history")]
mod history;
#[cfg(feature = "history")]
mod history_tool;
mod host;
mod progress;
#[cfg(feature = "urlshorten")]
mod urlshorten;
mod util;

use std::process;

use crate::action::debug::Debug;
use crate::action::delete::Delete;
use crate::action::download::Download;
use crate::action::exists::Exists;
use crate::action::generate::Generate;
#[cfg(feature = "history")]
use crate::action::history::History;
use crate::action::info::Info;
use crate::action::params::Params;
use crate::action::password::Password;
use crate::action::upload::Upload;
use crate::action::version::Version;
use crate::cmd::{
    matcher::{MainMatcher, Matcher},
    Handler,
};
use crate::error::Error;
use crate::util::{bin_name, highlight, quit_error, ErrorHints};

/// Application entrypoint.
fn main() {
    // Probe for OpenSSL certificates
    openssl_probe::init_ssl_cert_env_vars();

    // Parse CLI arguments
    let cmd_handler = Handler::parse();

    // Invoke the proper action
    if let Err(err) = invoke_action(&cmd_handler) {
        quit_error(err, ErrorHints::default());
    };
}

/// Invoke the proper action based on the CLI input.
///
/// If no proper action is selected, the program will quit with an error
/// message.
fn invoke_action(handler: &Handler) -> Result<(), Error> {
    // Match the debug command
    if handler.debug().is_some() {
        return Debug::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Match the delete command
    if handler.delete().is_some() {
        return Delete::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Match the download command
    if handler.download().is_some() {
        return Download::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Match the exists command
    if handler.exists().is_some() {
        return Exists::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Match the generate command
    if handler.generate().is_some() {
        return Generate::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Match the history command
    #[cfg(feature = "history")]
    {
        if handler.history().is_some() {
            return History::new(handler.matches())
                .invoke()
                .map_err(|err| err.into());
        }
    }

    // Match the info command
    if handler.info().is_some() {
        return Info::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Match the parameters command
    if handler.params().is_some() {
        return Params::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Match the password command
    if handler.password().is_some() {
        return Password::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Match the upload command
    if handler.upload().is_some() {
        return Upload::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Match the version command
    if handler.version().is_some() {
        return Version::new(handler.matches())
            .invoke()
            .map_err(|err| err.into());
    }

    // Get the main matcher
    let matcher_main = MainMatcher::with(handler.matches()).unwrap();

    // Print the main info and return
    if !matcher_main.quiet() {
        print_main_info();
    }
    Ok(())
}

/// Print the main info, shown when no subcommands were supplied.
pub fn print_main_info() -> ! {
    // Get the name of the used executable
    let bin = bin_name();

    // Print the main info
    println!("{} {}", crate_name!(), crate_version!());
    println!("Usage: {} [FLAGS] <SUBCOMMAND> ...", bin);
    println!();
    println!(crate_description!());
    println!();
    println!("Missing subcommand. Here are the most used:");
    println!("    {}", highlight(&format!("{} upload <FILE> ...", bin)));
    println!("    {}", highlight(&format!("{} download <URL> ...", bin)));
    println!();
    println!("To show all subcommands, features and other help:");
    println!("    {}", highlight(&format!("{} help [SUBCOMMAND]", bin)));
    println!();
    println!("The default public Send host is provided by Tim Visee.");
    println!("Please consider to donate and help keep it running: https://vis.ee/donate");

    process::exit(1)
}
