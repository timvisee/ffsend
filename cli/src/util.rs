#[cfg(feature = "clipboard")]
extern crate clipboard;
extern crate open;

#[cfg(feature = "clipboard")]
use std::error::Error;
use std::io::Error as IoError;
use std::process::{exit, ExitStatus};

#[cfg(feature = "clipboard")]
use self::clipboard::{ClipboardContext, ClipboardProvider};
use ffsend_api::url::Url;

/// Quit the application with an error code,
/// and print the given error message.
pub fn quit_error<S: AsRef<str>>(err: S) -> ! {
    // Print the error message
    eprintln!("error: {}", err.as_ref());

    // Quit
    exit(1);
}

/// Open the given URL in the users default browser.
/// The browsers exit statis is returned.
pub fn open_url(url: Url) -> Result<ExitStatus, IoError> {
    open_path(url.as_str())
}

/// Open the given path or URL using the program configured on the system.
/// The program exit statis is returned.
pub fn open_path(path: &str) -> Result<ExitStatus, IoError> {
    open::that(path)
}

/// Set the clipboard of the user to the given `content` string.
#[cfg(feature = "clipboard")]
pub fn set_clipboard(content: String) -> Result<(), Box<Error>> {
    let mut context: ClipboardContext = ClipboardProvider::new()?;
    context.set_contents(content)
}
