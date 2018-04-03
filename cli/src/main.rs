extern crate clap;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate ffsend_api;
extern crate rpassword;

mod action;
mod app;
mod cmd;
mod error;
mod progress;
mod util;

use action::delete::Delete;
use action::download::Download;
use action::info::Info;
use action::params::Params;
use action::password::Password;
use action::upload::Upload;
use cmd::Handler;
use error::Error;
use util::quit_error;

/// Application entrypoint.
fn main() {
    // Parse CLI arguments
    let cmd_handler = Handler::parse();

    // Invoke the proper action
    if let Err(err) = invoke_action(&cmd_handler) {
        quit_error(err);
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

    // No subcommand was selected, show general help
    Handler::build()
        .print_help()
        .expect("failed to print command help");

    Ok(())
}
