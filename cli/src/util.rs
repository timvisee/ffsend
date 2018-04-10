#[cfg(feature = "clipboard")]
extern crate clipboard;
extern crate colored;
extern crate open;

#[cfg(feature = "clipboard")]
use std::error::Error as StdError;
use std::fmt::{Debug, Display};
use std::io::Error as IoError;
use std::process::{exit, ExitStatus};

#[cfg(feature = "clipboard")]
use self::clipboard::{ClipboardContext, ClipboardProvider};
use self::colored::*;
use failure::{self, Fail};
use ffsend_api::url::Url;
use rpassword::prompt_password_stderr;

/// Print a success message.
pub fn print_success(msg: &str) {
    println!("{}", msg.green());
}

/// Print the given error in a proper format for the user,
/// with it's causes.
pub fn print_error<E: Fail>(err: E) {
    // Report each printable error, count them
    let count = err.causes() .map(|err| format!("{}", err))
        .filter(|err| !err.is_empty())
        .enumerate()
        .map(|(i, err)| if i == 0 {
            eprintln!("{} {}", "error:".red().bold(), err);
        } else {
            eprintln!("{} {}", "caused by:".red().bold(), err);
        })
        .count();

    // Fall back to a basic message
    if count == 0 {
        eprintln!("{} {}", "error:".red().bold(), "An undefined error occurred");
    }
}

/// Quit the application with an error code,
/// and print the given error.
pub fn quit_error<E: Fail>(err: E) -> ! {
    // Print the error
    print_error(err);

    // Print some additional information
    eprintln!("\nFor detailed errors try '{}'", "--verbose".yellow());
    eprintln!("For more information try '{}'", "--help".yellow());

    // Quit
    exit(1);
}

/// Quit the application with an error code,
/// and print the given error message.
pub fn quit_error_msg<S>(err: S) -> !
    where
        S: AsRef<str> + Display + Debug + Sync + Send + 'static
{
    quit_error(failure::err_msg(err).compat());
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

/// Prompt the user to enter a password.
pub fn prompt_password() -> String {
    match prompt_password_stderr("Password: ") {
        Ok(password) => password,
        Err(err) => quit_error(err.context(
            "Failed to read password from stdin with password prompt"
        )),
    }
}
