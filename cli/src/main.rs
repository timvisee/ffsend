#[macro_use]
extern crate clap;
#[macro_use]
extern crate derive_builder;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate ffsend_api;
extern crate rpassword;

mod action;
mod cmd;
mod error;
mod host;
mod progress;
mod util;

use action::delete::Delete;
use action::download::Download;
use action::exists::Exists;
use action::info::Info;
use action::params::Params;
use action::password::Password;
use action::upload::Upload;
use cmd::Handler;
use error::Error;
use util::{ErrorHints, exe_name, highlight, quit_error};

/// Application entrypoint.
fn main() {
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
    // Match the delete command
    if handler.delete().is_some() {
        return Delete::new(handler.matches()).invoke()
            .map_err(|err| err.into());
    }

    // Match the download command
    if handler.download().is_some() {
        return Download::new(handler.matches()).invoke()
            .map_err(|err| err.into());
    }

    // Match the exists command
    if handler.exists().is_some() {
        return Exists::new(handler.matches()).invoke()
            .map_err(|err| err.into());
    }

    // Match the info command
    if handler.info().is_some() {
        return Info::new(handler.matches()).invoke()
            .map_err(|err| err.into());
    }

    // Match the parameters command
    if handler.params().is_some() {
        return Params::new(handler.matches()).invoke()
            .map_err(|err| err.into());
    }

    // Match the password command
    if handler.password().is_some() {
        return Password::new(handler.matches()).invoke()
            .map_err(|err| err.into());
    }

    // Match the upload command
    if handler.upload().is_some() {
        return Upload::new(handler.matches()).invoke()
            .map_err(|err| err.into());
    }

    // Print the main info and return
    print_main_info();
    Ok(())
}

/// Print the main info, shown when no subcommands were supplied.
pub fn print_main_info() {
    // Get the name of the used executable
    let exe = exe_name();

    // Print the main info
    println!("{} {}", crate_name!(), crate_version!());
    println!("Usage: {} [FLAGS] <SUBCOMMAND> ...", exe);
    println!("");
    println!("{}", crate_description!());
    println!("");
    println!("Missing subcommand. Here are the most used:");
    println!("    {}", highlight(&format!("{} upload <FILE> ...", exe)));
    println!("    {}", highlight(&format!("{} download <URL> ...", exe)));
    println!("");
    println!("To show all subcommands, features and other help:");
    println!("    {}", highlight(&format!("{} help [SUBCOMMAND]", exe)));
}
