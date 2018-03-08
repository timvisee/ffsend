use std::process::exit;

/// Quit the application with an error code,
/// and print the given error message.
pub fn quit_error<S: AsRef<str>>(err: S) -> ! {
    // Print the error message
    eprintln!("error: {}", err.as_ref());

    // Quit
    exit(1);
}
