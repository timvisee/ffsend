#[cfg(feature = "clipboard")]
extern crate clipboard;
extern crate failure;
extern crate open;

#[cfg(feature = "clipboard")]
use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::io::Error as IoError;
use std::process::{exit, ExitStatus};

#[cfg(feature = "clipboard")]
use self::clipboard::{ClipboardContext, ClipboardProvider};
use self::failure::{Fail};
use ffsend_api::url::Url;

/// Quit the application with an error code,
/// and print the given error.
pub fn quit_error<E: Fail>(err: E) -> ! {
    // Print the error message
    eprintln!("error: {}", err);

    // Quit
    exit(1);
}

/// Quit the application with an error code,
/// and print the given error message.
pub fn quit_error_msg<S>(err: S) -> !
    where
        S: AsRef<str> + Display + Debug + Sync + Send + 'static
{
    // TODO: forward the error the `quit_error` here
    // quit_error(failure::err_msg(err));

    // Print the error message
    eprintln!("error: {}", err);

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
pub fn set_clipboard(content: String) -> Result<(), Box<StdError>> {
    let mut context: ClipboardContext = ClipboardProvider::new()?;
    context.set_contents(content)
}
