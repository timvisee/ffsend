use failure::Fail;
use ffsend_api::file::remote_file::RemoteFile;

use cmd::matcher::MainMatcher;
use history::{
    Error as HistoryError,
    History,
};
use util::print_error;

/// Load the history from the given path, add the given file, and save it
/// again.
///
/// When a file with the same ID already exists, the existing file is
/// merged with this one. If `overwrite` is set to true, this file will
/// overwrite properties in the already existing file when merging.
///
/// If there is no file at the given path, new history will be created.
fn add_error(matcher_main: &MainMatcher, file: RemoteFile, overwrite: bool)
    -> Result<(), HistoryError>
{
    // Ignore if incognito
    if matcher_main.incognito() {
        return Ok(());
    }

    // Load the history, add the file, and save
    let mut history = History::load_or_new(matcher_main.history())?;
    history.add(file, overwrite);
    history.save().map_err(|err| err.into())
}

/// Load the history from the given path, add the given file, and save it
/// again.
/// If there is no file at the given path, new history will be created.
///
/// When a file with the same ID already exists, the existing file is
/// merged with this one. If `overwrite` is set to true, this file will
/// overwrite properties in the already existing file when merging.
///
/// If an error occurred, the error is printed and ignored.
pub fn add(matcher_main: &MainMatcher, file: RemoteFile, overwrite: bool) {
    if let Err(err) = add_error(matcher_main, file, overwrite) {
        print_error(err.context(
            "Failed to add file to local history, ignoring",
        ));
    }
}

/// Load the history from the given path, remove the given file by it's
/// ID, and save it again.
/// True is returned if any file was removed.
fn remove_error(matcher_main: &MainMatcher, file: &RemoteFile)
    -> Result<bool, HistoryError>
{
    // Ignore if incognito
    if matcher_main.incognito() {
        return Ok(false);
    }

    // Load the history, remove the file, and save
    let mut history = History::load_or_new(matcher_main.history())?;
    let removed = history.remove(file);
    history.save()?;
    Ok(removed)
}

/// Load the history from the given path, remove the given file by it's
/// ID, and save it again.
/// True is returned if any file was removed.
pub fn remove(matcher_main: &MainMatcher, file: &RemoteFile) -> bool {
    let result = remove_error(matcher_main, file);
    let ok = result.is_ok();
    if let Err(err) = result {
        print_error(err.context(
            "Failed to remove file from local history, ignoring",
        ));
    }
    ok
}

/// Derive an owner token of a remote file from the current history.
/// The newly derived owner token will be set into the given borrowed remote file.
/// This method may be used to automatically derive the owner token for some file actions
/// requiring this token, to prevent the user from having to enter it manually.
///
/// If the file already has an owner token set,
/// nothing will be derived and `false` is returned.
/// If an error occurred while deriving,
/// the error is printed and `false` is returned.
/// If there was no matching file in the history,
/// and no owner token could be derived, `false` is returned.
/// Incognito mode does not have any effect on this method,
/// as it won't ever change the history.
///
/// If an owner token was successfully derived from the history,
/// `true` is returned.
///
/// Note, to check if an owner token is set in the remote file,
/// use the `file.has_owner_token()` method instead of the returned boolean from this method.
pub fn derive_owner_token(matcher_main: &MainMatcher, file: &mut RemoteFile) -> bool {
    // Return if the remote file already has an owner token set, or if we're incognito
    if file.has_owner_token() {
        return false;
    }

    // Load the history
    let history = match History::load_or_new(matcher_main.history()) {
        Ok(history) => history,
        Err(err) => {
            print_error(err.context(
                "Failed to derive file owner token from history, ignoring",
            ));
            return false;
        }
    };

    // Find a matching file, grab and set the owner token if available
    match history.get_file(file) {
        Some(f) if f.has_owner_token() => {
            file.set_owner_token(f.owner_token().cloned());
            println!("DEBUG: Owner token set from history");
            return true;
        },
        _ => return false,
    }
}
