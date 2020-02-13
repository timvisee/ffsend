extern crate colored;
extern crate directories;
extern crate fs2;
extern crate open;
extern crate regex;

use std::borrow::Borrow;
use std::env::{self, current_exe, var_os};
#[cfg(feature = "clipboard")]
use std::error::Error as StdError;
use std::ffi::OsStr;
use std::fmt::{Debug, Display};
use std::io::{stderr, stdin, Error as IoError, Write};
use std::path::Path;
use std::path::PathBuf;
use std::process::{exit, ExitStatus};

use self::colored::*;
#[cfg(feature = "history")]
use self::directories::ProjectDirs;
use self::fs2::available_space;
use chrono::Duration;
#[cfg(feature = "clipboard")]
use clipboard_ext::{prelude::*, x11_bin::ClipboardContext};
use failure::{err_msg, Fail};
use ffsend_api::{
    api::request::{ensure_success, ResponseError},
    client::Client,
    reqwest,
    url::Url,
};
use regex::Regex;
use rpassword::prompt_password_stderr;

use crate::cmd::matcher::MainMatcher;

/// Print a success message.
pub fn print_success(msg: &str) {
    eprintln!("{}", msg.green());
}

/// Print the given error in a proper format for the user,
/// with it's causes.
pub fn print_error<E: Fail>(err: impl Borrow<E>) {
    // Report each printable error, count them
    let count = err
        .borrow()
        .causes()
        .map(|err| format!("{}", err))
        .filter(|err| !err.is_empty())
        .enumerate()
        .map(|(i, err)| {
            if i == 0 {
                eprintln!("{} {}", highlight_error("error:"), err);
            } else {
                eprintln!("{} {}", highlight_error("caused by:"), err);
            }
        })
        .count();

    // Fall back to a basic message
    if count == 0 {
        eprintln!(
            "{} {}",
            highlight_error("error:"),
            "an undefined error occurred"
        );
    }
}

/// Print the given error message in a proper format for the user,
/// with it's causes.
pub fn print_error_msg<S>(err: S)
where
    S: AsRef<str> + Display + Debug + Sync + Send + 'static,
{
    print_error(err_msg(err).compat());
}

/// Print a warning.
pub fn print_warning<S>(err: S)
where
    S: AsRef<str> + Display + Debug + Sync + Send + 'static,
{
    eprintln!("{} {}", highlight_warning("warning:"), err);
}

/// Quit the application regularly.
pub fn quit() -> ! {
    exit(0);
}

/// Quit the application with an error code,
/// and print the given error.
pub fn quit_error<E: Fail>(err: E, hints: impl Borrow<ErrorHints>) -> ! {
    // Print the error
    print_error(err);

    // Print error hints
    hints.borrow().print();

    // Quit
    exit(1);
}

/// Quit the application with an error code,
/// and print the given error message.
pub fn quit_error_msg<S>(err: S, hints: impl Borrow<ErrorHints>) -> !
where
    S: AsRef<str> + Display + Debug + Sync + Send + 'static,
{
    quit_error(err_msg(err).compat(), hints);
}

/// The error hint configuration.
#[derive(Clone, Builder)]
#[builder(default)]
pub struct ErrorHints {
    /// Show about specifying an API version.
    api: bool,

    /// A list of info messages to print along with the error.
    info: Vec<String>,

    /// Show about the name option.
    name: bool,

    /// Show about the password option.
    password: bool,

    /// Show about the owner option.
    owner: bool,

    /// Show about the history flag.
    #[cfg(feature = "history")]
    history: bool,

    /// Show about the force flag.
    force: bool,

    /// Show about the verbose flag.
    verbose: bool,

    /// Show about the help flag.
    help: bool,
}

impl ErrorHints {
    /// Check whether any hint should be printed.
    pub fn any(&self) -> bool {
        // Determine the result
        #[allow(unused_mut)]
        let mut result =
            self.name || self.password || self.owner || self.force || self.verbose || self.help;

        // Factor in the history hint when enabled
        #[cfg(feature = "history")]
        {
            result = result || self.history;
        }

        result
    }

    /// Print the error hints.
    pub fn print(&self) {
        // Print info messages
        for msg in &self.info {
            eprintln!("{} {}", highlight_info("info:"), msg);
        }

        // Stop if nothing should be printed
        if !self.any() {
            return;
        }

        eprint!("\n");

        // Print hints
        if self.api {
            eprintln!(
                "Use '{}' to select a server API version",
                highlight("--api <VERSION>")
            );
        }
        if self.name {
            eprintln!(
                "Use '{}' to specify a file name",
                highlight("--name <NAME>")
            );
        }
        if self.password {
            eprintln!(
                "Use '{}' to specify a password",
                highlight("--password <PASSWORD>")
            );
        }
        if self.owner {
            eprintln!(
                "Use '{}' to specify an owner token",
                highlight("--owner <TOKEN>")
            );
        }
        #[cfg(feature = "history")]
        {
            if self.history {
                eprintln!(
                    "Use '{}' to specify a history file",
                    highlight("--history <FILE>")
                );
            }
        }
        if self.force {
            eprintln!("Use '{}' to force", highlight("--force"));
        }
        if self.verbose {
            eprintln!("For detailed errors try '{}'", highlight("--verbose"));
        }
        if self.help {
            eprintln!("For more information try '{}'", highlight("--help"));
        }

        // Flush
        let _ = stderr().flush();
    }
}

impl Default for ErrorHints {
    fn default() -> Self {
        ErrorHints {
            api: false,
            info: Vec::new(),
            name: false,
            password: false,
            owner: false,
            #[cfg(feature = "history")]
            history: false,
            force: false,
            verbose: true,
            help: true,
        }
    }
}

impl ErrorHintsBuilder {
    /// Add a single info entry.
    pub fn add_info(mut self, info: String) -> Self {
        // Initialize the info list
        if self.info.is_none() {
            self.info = Some(Vec::new());
        }

        // Add the item to the info list
        if let Some(ref mut list) = self.info {
            list.push(info);
        }

        self
    }
}

/// Highlight the given text with a color.
pub fn highlight(msg: &str) -> ColoredString {
    msg.yellow()
}

/// Highlight the given text with an error color.
pub fn highlight_error(msg: &str) -> ColoredString {
    msg.red().bold()
}

/// Highlight the given text with an warning color.
pub fn highlight_warning(msg: &str) -> ColoredString {
    highlight(msg).bold()
}

/// Highlight the given text with an info color
pub fn highlight_info(msg: &str) -> ColoredString {
    msg.cyan()
}

/// Open the given URL in the users default browser.
/// The browsers exit statis is returned.
pub fn open_url(url: impl Borrow<Url>) -> Result<ExitStatus, IoError> {
    open_path(url.borrow().as_str())
}

/// Open the given path or URL using the program configured on the system.
/// The program exit statis is returned.
pub fn open_path(path: &str) -> Result<ExitStatus, IoError> {
    open::that(path)
}

/// Set the clipboard of the user to the given `contents` string.
#[cfg(feature = "clipboard")]
pub fn set_clipboard(contents: String) -> Result<(), Box<dyn StdError>> {
    Ok(ClipboardContext::new()?.set_contents(contents)?)
}

/// Check for an emtpy password in the given `password`.
/// If the password is emtpy the program will quit with an error unless
/// forced.
// TODO: move this to a better module
pub fn check_empty_password(password: &str, matcher_main: &MainMatcher) {
    if !matcher_main.force() && password.is_empty() {
        quit_error_msg(
            "an empty password is not supported by the web interface",
            ErrorHintsBuilder::default()
                .force(true)
                .verbose(false)
                .build()
                .unwrap(),
        )
    }
}

/// Prompt the user to enter a password.
///
/// If `empty` is `false`, emtpy passwords aren't allowed unless forced.
pub fn prompt_password(main_matcher: &MainMatcher, optional: bool) -> Option<String> {
    // Quit with an error if we may not interact
    if !optional && main_matcher.no_interact() {
        quit_error_msg(
            "missing password, must be specified in no-interact mode",
            ErrorHintsBuilder::default()
                .password(true)
                .verbose(false)
                .build()
                .unwrap(),
        );
    }

    // Prompt for the password
    let prompt = if optional {
        "Password (optional): "
    } else {
        "Password: "
    };
    match prompt_password_stderr(prompt) {
        // If optional and nothing is entered, regard it as not defined
        Ok(password) => {
            if password.is_empty() && optional {
                None
            } else {
                Some(password)
            }
        }

        // On input error, propegate the error or don't use a password if optional
        Err(err) => {
            if !optional {
                quit_error(
                    err.context("failed to read password from password prompt"),
                    ErrorHints::default(),
                )
            } else {
                None
            }
        }
    }
}

/// Get a password if required.
/// This method will ensure a password is set (or not) in the given `password`
/// parameter, as defined by `needs`.
/// If a password is needed, it may optionally be entered if `option` is set to true.
///
/// This method will prompt the user for a password, if one is required but
/// wasn't set. An ignore message will be shown if it was not required while it
/// was set.
///
/// Returns true if a password is now set, false if not.
pub fn ensure_password(
    password: &mut Option<String>,
    needs: bool,
    main_matcher: &MainMatcher,
    optional: bool,
) -> bool {
    // Return if we're fine, ignore if set but we don't need it
    if password.is_some() == needs {
        return needs;
    }
    if !needs {
        // Notify the user a set password is ignored
        if password.is_some() {
            println!("Ignoring password, it is not required");
            *password = None;
        }
        return false;
    }

    // Check whehter we allow interaction
    let interact = !main_matcher.no_interact();

    loop {
        // Prompt for an owner token if not set yet
        if password.is_none() {
            // Do not ask for a token if optional when non-interactive or forced
            if optional && (!interact || main_matcher.force()) {
                return false;
            }

            // Ask for the password
            *password = prompt_password(main_matcher, optional);
        }

        // The token must not be empty, unless it's optional
        let empty = password.is_none();
        if empty && !optional {
            eprintln!(
                "No password given, which is required. Use {} to cancel.",
                highlight("[CTRL+C]"),
            );
        } else {
            return !empty;
        }
    }
}

/// Prompt the user to enter some value.
/// The prompt that is shown should be passed to `msg`,
/// excluding the `:` suffix.
pub fn prompt(msg: &str, main_matcher: &MainMatcher) -> String {
    // Quit with an error if we may not interact
    if main_matcher.no_interact() {
        quit_error_msg(
            format!(
                "could not prompt for '{}' in no-interact mode, maybe specify it",
                msg,
            ),
            ErrorHints::default(),
        );
    }

    // Show the prompt
    eprint!("{}: ", msg);
    let _ = stderr().flush();

    // Get the input
    let mut input = String::new();
    if let Err(err) = stdin().read_line(&mut input) {
        quit_error(
            err.context("failed to read input from prompt"),
            ErrorHints::default(),
        );
    }

    // Trim and return
    input.trim().to_owned()
}

/// Prompt the user for a question, allowing a yes or now answer.
/// True is returned if yes was answered, false if no.
///
/// A default may be given, which is chosen if no-interact mode is
/// enabled, or if enter was pressed by the user without entering anything.
pub fn prompt_yes(msg: &str, def: Option<bool>, main_matcher: &MainMatcher) -> bool {
    // Define the available options string
    let options = format!(
        "[{}/{}]",
        match def {
            Some(def) if def => "Y",
            _ => "y",
        },
        match def {
            Some(def) if !def => "N",
            _ => "n",
        }
    );

    // Assume yes
    if main_matcher.assume_yes() {
        eprintln!("{} {}: yes", msg, options);
        return true;
    }

    // Autoselect if in no-interact mode
    if main_matcher.no_interact() {
        if let Some(def) = def {
            eprintln!("{} {}: {}", msg, options, if def { "yes" } else { "no" });
            return def;
        } else {
            quit_error_msg(
                format!(
                    "could not prompt question '{}' in no-interact mode, maybe specify it",
                    msg,
                ),
                ErrorHints::default(),
            );
        }
    }

    // Get the user input
    let answer = prompt(&format!("{} {}", msg, options), main_matcher);

    // Assume the default if the answer is empty
    if answer.is_empty() && def.is_some() {
        return def.unwrap();
    }

    // Derive a boolean and return
    match derive_bool(&answer) {
        Some(answer) => answer,
        None => prompt_yes(msg, def, main_matcher),
    }
}

/// Try to derive true or false (yes or no) from the given input.
/// None is returned if no boolean could be derived accurately.
fn derive_bool(input: &str) -> Option<bool> {
    // Process the input
    let input = input.trim().to_lowercase();

    // Handle short or incomplete answers
    match input.as_str() {
        "y" | "ye" | "t" | "1" => return Some(true),
        "n" | "f" | "0" => return Some(false),
        _ => {}
    }

    // Handle complete answers with any suffix
    if input.starts_with("yes") || input.starts_with("true") {
        return Some(true);
    }
    if input.starts_with("no") || input.starts_with("false") {
        return Some(false);
    }

    // The answer could not be determined, return none
    None
}

/// Prompt the user to enter an owner token.
pub fn prompt_owner_token(main_matcher: &MainMatcher, optional: bool) -> String {
    prompt(
        if optional {
            "Owner token (optional)"
        } else {
            "Owner token"
        },
        main_matcher,
    )
}

/// Get the owner token.
/// This method will ensure an owner token is set in the given `token`
/// parameter.
///
/// This method will prompt the user for the token, if it wasn't set.
///
/// Returns if an owner token was set.
/// If `optional` is false, this always returns true.
///
/// If in non-interactive or force mode, the user will not be prompted for a token if `optional` is
/// set to true.
pub fn ensure_owner_token(
    token: &mut Option<String>,
    main_matcher: &MainMatcher,
    optional: bool,
) -> bool {
    // Check whehter we allow interaction
    let interact = !main_matcher.no_interact();

    // Notify that an owner token is required
    if interact && token.is_none() {
        if optional {
            println!("The file owner token is recommended for authentication.");
        } else {
            println!("The file owner token is required for authentication.");
        }
    }

    loop {
        // Prompt for an owner token if not set yet
        if token.is_none() {
            // Do not ask for a token if optional when non-interactive or forced
            if optional && (!interact || main_matcher.force()) {
                return false;
            }

            // Ask for the token, or quit with an error if non-interactive
            if interact {
                *token = Some(prompt_owner_token(main_matcher, optional));
            } else {
                quit_error_msg(
                    "missing owner token, must be specified in no-interact mode",
                    ErrorHintsBuilder::default()
                        .owner(true)
                        .verbose(false)
                        .build()
                        .unwrap(),
                );
            }
        }

        // The token must not be empty, unless it's optional
        let empty = token.as_ref().unwrap().is_empty();
        if empty {
            *token = None;
        }
        if empty && !optional {
            eprintln!(
                "Empty owner token given, which is invalid. Use {} to cancel.",
                highlight("[CTRL+C]"),
            );
        } else {
            return !empty;
        }
    }
}

/// Format the given number of bytes readable for humans.
pub fn format_bytes(bytes: u64) -> String {
    let bytes = bytes as f64;
    let kb = 1024f64;
    match bytes {
        bytes if bytes >= kb.powf(4_f64) => format!("{:.*} TiB", 2, bytes / kb.powf(4_f64)),
        bytes if bytes >= kb.powf(3_f64) => format!("{:.*} GiB", 2, bytes / kb.powf(3_f64)),
        bytes if bytes >= kb.powf(2_f64) => format!("{:.*} MiB", 2, bytes / kb.powf(2_f64)),
        bytes if bytes >= kb => format!("{:.*} KiB", 2, bytes / kb),
        _ => format!("{:.*} B", 0, bytes),
    }
}

/// Parse the given duration string from human readable format into seconds.
/// This method parses a string of time components to represent the given duration.
///
/// The following time units are used:
/// - `w`: weeks
/// - `d`: days
/// - `h`: hours
/// - `m`: minutes
/// - `s`: seconds
/// The following time strings can be parsed:
/// - `8w6d`
/// - `23h14m`
/// - `9m55s`
/// - `1s1s1s1s1s`
pub fn parse_duration(duration: &str) -> Result<usize, ParseDurationError> {
    // Build a regex to grab time parts
    let re = Regex::new(r"(?i)([0-9]+)(([a-z]|\s*$))")
        .expect("failed to compile duration parsing regex");

    // We must find any match
    if re.find(duration).is_none() {
        return Err(ParseDurationError::Empty);
    }

    // Parse each time part, sum it's seconds
    let mut seconds = 0;
    for capture in re.captures_iter(duration) {
        // Parse time value and modifier
        let number = capture[1]
            .parse::<usize>()
            .map_err(ParseDurationError::InvalidValue)?;
        let modifier = capture[2].trim().to_lowercase();

        // Multiply and sum seconds by modifier
        seconds += match modifier.as_str() {
            "" | "s" => number,
            "m" => number * 60,
            "h" => number * 60 * 60,
            "d" => number * 60 * 60 * 24,
            "w" => number * 60 * 60 * 24 * 7,
            m => return Err(ParseDurationError::UnknownIdentifier(m.into())),
        };
    }

    Ok(seconds)
}

/// Represents a duration parsing error.
#[derive(Debug, Fail)]
pub enum ParseDurationError {
    /// The given duration string did not contain any duration part.
    #[fail(display = "given string did not contain any duration part")]
    Empty,

    /// A numeric value was invalid.
    #[fail(display = "duration part has invalid numeric value")]
    InvalidValue(std::num::ParseIntError),

    /// The given duration string contained an invalid duration modifier.
    #[fail(display = "duration part has unknown time identifier '{}'", _0)]
    UnknownIdentifier(String),
}

/// Format the given duration in a human readable format.
/// This method builds a string of time components to represent
/// the given duration.
///
/// The following time units are used:
/// - `w`: weeks
/// - `d`: days
/// - `h`: hours
/// - `m`: minutes
/// - `s`: seconds
///
/// Only the two most significant units are returned.
/// If the duration is zero seconds or less `now` is returned.
///
/// The following time strings may be produced:
/// - `8w6d`
/// - `23h14m`
/// - `9m55s`
/// - `1s`
/// - `now`
pub fn format_duration(duration: impl Borrow<Duration>) -> String {
    // Get the total number of seconds, return immediately if zero or less
    let mut secs = duration.borrow().num_seconds();
    if secs <= 0 {
        return "now".into();
    }

    // Build a list of time units, define a list for time components
    let mut components = Vec::new();
    let units = [
        (60 * 60 * 24 * 7, "w"),
        (60 * 60 * 24, "d"),
        (60 * 60, "h"),
        (60, "m"),
        (1, "s"),
    ];

    // Fill the list of time components based on the units which fit
    for unit in &units {
        if secs >= unit.0 {
            components.push(format!("{}{}", secs / unit.0, unit.1));
            secs %= unit.0;
        }
    }

    // Show only the two most significant components and join them in a string
    components.truncate(2);
    components.join("")
}

/// Format the given boolean, as `yes` or `no`.
pub fn format_bool(b: bool) -> &'static str {
    if b {
        "yes"
    } else {
        "no"
    }
}

/// Get the name of the executable that was invoked.
///
/// When a symbolic or hard link is used, the name of the link is returned.
///
/// This attempts to obtain the binary name in the following order:
/// - name in first item of program arguments via `std::env::args`
/// - current executable name via `std::env::current_exe`
/// - crate name
pub fn bin_name() -> String {
    env::args_os()
        .next()
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| current_exe().ok())
        .and_then(|p| p.file_name().map(|n| n.to_owned()))
        .and_then(|n| n.into_string().ok())
        .unwrap_or_else(|| crate_name!().into())
}

/// Ensure that there is enough free disk space available at the given `path`,
/// to store a file with the given `size`.
///
/// If an error occurred while querying the file system,
/// the error is reported to the user and the method returns.
///
/// If there is not enough disk space available,
/// an error is reported and the program will quit.
pub fn ensure_enough_space<P: AsRef<Path>>(path: P, size: u64) {
    // Get the available space at this path
    let space = match available_space(path) {
        Ok(space) => space,
        Err(err) => {
            print_error(err.context("failed to check available space on disk, ignoring"));
            return;
        }
    };

    // Return if enough disk space is avaiable
    if space >= size {
        return;
    }

    // Create an info message giving details about the required space
    let info = format!(
        "{} of space required, but only {} is available",
        format_bytes(size),
        format_bytes(space),
    );

    // Print an descriptive error and quit
    quit_error(
        err_msg("not enough disk space available in the target directory")
            .context("failed to download file"),
        ErrorHintsBuilder::default()
            .add_info(info)
            .force(true)
            .verbose(false)
            .build()
            .unwrap(),
    );
}

/// Get the project directories instance for this application.
/// This may be used to determine the project, cache, configuration, data and
/// some other directory paths.
#[cfg(feature = "history")]
pub fn app_project_dirs() -> ProjectDirs {
    ProjectDirs::from("", "", crate_name!())
        .expect("failed to determine location of project directories")
}

/// Get the default path to use for the history file.
#[cfg(feature = "history")]
pub fn app_history_file_path() -> PathBuf {
    app_project_dirs().cache_dir().join("history.toml")
}

/// Get the default path to use for the history file, as a string.
#[cfg(feature = "history")]
pub fn app_history_file_path_string() -> String {
    app_history_file_path().to_str().unwrap().to_owned()
}

/// Check whether an environment variable with the given key is present in the context of the
/// current process. The environment variable doesn't have to hold any specific value.
/// Returns `true` if present, `false` if not.
pub fn env_var_present(key: impl AsRef<OsStr>) -> bool {
    var_os(key).is_some()
}

/// Get a list of all features that were enabled during compilation.
pub fn features_list() -> Vec<&'static str> {
    // Build the list
    #[allow(unused_mut)]
    let mut features = Vec::new();

    // Add each feature
    #[cfg(feature = "archive")]
    features.push("archive");
    #[cfg(feature = "clipboard")]
    features.push("clipboard");
    #[cfg(feature = "history")]
    features.push("history");
    #[cfg(feature = "qrcode")]
    features.push("qrcode");
    #[cfg(feature = "urlshorten")]
    features.push("urlshorten");
    #[cfg(feature = "infer-command")]
    features.push("infer-command");
    #[cfg(feature = "no-qcolor")]
    features.push("no-color");
    #[cfg(feature = "send2")]
    features.push("send2");
    #[cfg(feature = "send3")]
    features.push("send3");

    features
}

/// Get a list of supported API versions.
pub fn api_version_list() -> Vec<&'static str> {
    // Build the list
    #[allow(unused_mut)]
    let mut versions = Vec::new();

    // Add each feature
    #[cfg(feature = "send2")]
    versions.push("v2");
    #[cfg(feature = "send3")]
    versions.push("v3");

    versions
}

/// Follow redirects on the given URL, and return the final full URL.
///
/// This is used to obtain share URLs from shortened links.
///
// TODO: extract this into module
pub fn follow_url(client: &Client, url: &Url) -> Result<Url, FollowError> {
    // Send the request, follow the URL, ensure success
    let response = client
        .get(url.as_str())
        .send()
        .map_err(FollowError::Request)?;
    ensure_success(&response)?;

    // Obtain the final URL
    Ok(response.url().clone())
}

/// URL following error.
#[derive(Debug, Fail)]
pub enum FollowError {
    /// Failed to send the shortening request.
    #[fail(display = "failed to send URL follow request")]
    Request(#[cause] reqwest::Error),

    /// The server responded with a bad response.
    #[fail(display = "failed to shorten URL, got bad response")]
    Response(#[cause] ResponseError),
}

impl From<ResponseError> for FollowError {
    fn from(err: ResponseError) -> Self {
        FollowError::Response(err)
    }
}
