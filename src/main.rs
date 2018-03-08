extern crate hyper;
extern crate mime_guess;
extern crate open;
extern crate openssl;
extern crate reqwest;
#[macro_use]
extern crate serde_derive;
extern crate url;

mod action;
mod app;
mod b64;
mod cmd;
mod crypto;
mod send;
mod util;

use cmd::Handler;
use cmd::cmd_upload::CmdUpload;

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
        return action_upload(&cmd);
    }

    // No subcommand was selected, show general help
    Handler::build()
        .print_help()
        .expect("failed to print command help");
}

/// The upload action.
fn action_upload(cmd_upload: &CmdUpload) {
    // // Get the path and host
    // let path = Path::new(cmd_upload.file());
    // let host = cmd_upload.host();

    // // Open the URL in the browser
    // open::that(url).expect("failed to open URL");
}
