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
