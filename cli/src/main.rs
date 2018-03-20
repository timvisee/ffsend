extern crate ffsend_api;

mod action;
mod app;
mod cmd;
mod progress;
mod util;

use action::download::Download;
use action::upload::Upload;
use cmd::Handler;

/// Application entrypoint.
fn main() {
    // Parse CLI arguments
    let cmd_handler = Handler::parse();

    // Invoke the proper action
    invoke_action(&cmd_handler);
}

/// Invoke the proper action based on the CLI input.
///
/// If no proper action is selected, the program will quit with an error
/// message.
fn invoke_action(handler: &Handler) {
    // Match the upload command
    if let Some(cmd) = handler.upload() {
        return Upload::new(&cmd).invoke();
    }

    // Match the download command
    if let Some(cmd) = handler.download() {
        return Download::new(&cmd).invoke();
    }

    // No subcommand was selected, show general help
    Handler::build()
        .print_help()
        .expect("failed to print command help");
}
