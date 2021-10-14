use std::fs;
use std::io::Error as IoError;
use std::path::PathBuf;

use failure::Fail;
use ffsend_api::{
    file::remote_file::{FileParseError, RemoteFile},
    url::Url,
};
use toml::{de::Error as DeError, ser::Error as SerError};
use version_compare::Cmp;

use crate::util::{print_error, print_warning};

/// The minimum supported history file version.
const VERSION_MIN: &str = "0.0.1";

/// The maximum supported history file version.
const VERSION_MAX: &str = crate_version!();

#[derive(Serialize, Deserialize)]
pub struct History {
    /// The application version the history file was built with.
    /// Used for compatibility checking.
    version: Option<String>,

    /// The file history.
    files: Vec<RemoteFile>,

    /// Whether the list of files has changed.
    #[serde(skip)]
    changed: bool,

    /// An optional path to automatically save the history to.
    #[serde(skip)]
    autosave: Option<PathBuf>,
}

impl History {
    /// Construct a new history.
    /// A path may be given to automatically save the history to once changed.
    pub fn new(autosave: Option<PathBuf>) -> Self {
        let mut history = History::default();
        history.autosave = autosave;
        history
    }

    /// Load the history from the given file.
    pub fn load(path: PathBuf) -> Result<Self, LoadError> {
        // Read the file to a string
        let data = fs::read_to_string(&path)?;

        // Parse the data, set the autosave path
        let mut history: Self = toml::from_str(&data)?;
        history.autosave = Some(path);

        // Make sure the file version is supported
        if history.version.is_none() {
            print_warning("History file has no version, ignoring");
            history.version = Some(crate_version!().into());
        } else {
            // Get the version number from the file
            let version = history.version.as_ref().unwrap();

            if let Ok(true) = version_compare::compare_to(version, VERSION_MIN, Cmp::Lt) {
                print_warning("history file version is too old, ignoring");
            } else if let Ok(true) = version_compare::compare_to(version, VERSION_MAX, Cmp::Gt) {
                print_warning("history file has an unknown version, ignoring");
            }
        }

        // Garbage collect
        history.gc();

        Ok(history)
    }

    /// Load the history from the given file.
    /// If the file doesn't exist, create a new empty history instance.
    ///
    /// Autosaving will be enabled, and will save to the given file path.
    pub fn load_or_new(file: PathBuf) -> Result<Self, LoadError> {
        if file.is_file() {
            Self::load(file)
        } else {
            Ok(Self::new(Some(file)))
        }
    }

    /// Save the history to the internal autosave file.
    pub fn save(&mut self) -> Result<(), SaveError> {
        // Garbage collect
        self.gc();

        // Get the path
        let path = self.autosave.as_ref().ok_or(SaveError::NoPath)?;

        // If we have no files, remove the history file if it exists
        if self.files.is_empty() {
            if path.is_file() {
                fs::remove_file(&path).map_err(SaveError::Delete)?;
            }
            return Ok(());
        }

        // Ensure the file parent directories are available
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Set file permissions on unix based systems
        #[cfg(unix)]
        {
            use std::fs::Permissions;
            use std::os::unix::fs::PermissionsExt;

            if !path.exists() {
                let file = fs::File::create(path).map_err(SaveError::Write)?;

                // Set Read/Write permissions for the user
                file.set_permissions(Permissions::from_mode(0o600))
                    .map_err(SaveError::SetPermissions)?;
            }
        }

        // Build the data and write to a file
        let data = toml::to_string(self)?;
        fs::write(&path, data)?;

        // There are no new changes, set the flag
        self.changed = false;

        Ok(())
    }

    /// Add the given remote file to the history.
    /// If a file with the same ID as the given file exists,
    /// the files are merged, see `RemoteFile::merge()`.
    ///
    /// If `overwrite` is set to true, the given file will overwrite
    /// properties on the existing file.
    pub fn add(&mut self, file: RemoteFile, overwrite: bool) {
        // Merge any existing file with the same ID
        {
            // Find anything to merge
            let merge_info: Vec<bool> = self
                .files
                .iter_mut()
                .filter(|f| f.id() == file.id())
                .map(|ref mut f| f.merge(&file, overwrite))
                .collect();
            let merged = !merge_info.is_empty();
            let changed = merge_info.iter().any(|i| *i);

            // Return if merged, update the changed state
            if merged {
                if changed {
                    self.changed = true;
                }
                return;
            }
        }

        // Add the file to the list
        self.files.push(file);
        self.changed = true;
    }

    /// Remove a file, matched by it's file ID.
    ///
    /// If any file was removed, true is returned.
    pub fn remove(&mut self, id: &str) -> bool {
        // Get the indices of files that have expired
        let expired_indices: Vec<usize> = self
            .files
            .iter()
            .enumerate()
            .filter(|&(_, f)| f.id() == id)
            .map(|(i, _)| i)
            .collect();

        // Remove these specific files
        for i in expired_indices.iter().rev() {
            self.files.remove(*i);
        }

        // Set the changed flag, and return
        if expired_indices.is_empty() {
            self.changed = true;
        }
        !expired_indices.is_empty()
    }

    /// Remove a file by the given URL.
    ///
    /// If any file was removed, true is returned.
    pub fn remove_url(&mut self, url: Url) -> Result<bool, FileParseError> {
        Ok(self.remove(RemoteFile::parse_url(url, None)?.id()))
    }

    /// Get all files.
    pub fn files(&self) -> &Vec<RemoteFile> {
        &self.files
    }

    /// Get a file from the history, based on the given remote file.
    /// The file ID and host will be compared against all files in this history.
    /// If multiple files exist within the history that are equal, only one is returned.
    /// If no matching file was found, `None` is returned.
    pub fn get_file(&self, file: &RemoteFile) -> Option<&RemoteFile> {
        self.files
            .iter()
            .find(|f| f.id() == file.id() && f.host() == file.host())
    }

    /// Clear all history.
    pub fn clear(&mut self) {
        self.changed = !self.files.is_empty();
        self.files.clear();
    }

    /// Garbage collect (remove) all files that have been expired,
    /// as defined by their `expire_at` property.
    ///
    /// If the expiry property is None (thus unknown), the file will be kept.
    ///
    /// The number of expired files is returned.
    pub fn gc(&mut self) -> usize {
        // Get a list of expired files
        let expired: Vec<RemoteFile> = self
            .files
            .iter()
            .filter(|f| f.has_expired())
            .cloned()
            .collect();

        // Remove the files
        for f in &expired {
            self.remove(f.id());
        }

        // Set the changed flag
        if !expired.is_empty() {
            self.changed = true;
        }

        // Return the number of expired files
        expired.len()
    }
}

impl Drop for History {
    fn drop(&mut self) {
        // Automatically save if enabled and something was changed
        if self.autosave.is_some() && self.changed {
            // Save and report errors
            if let Err(err) = self.save() {
                print_error(err.context("failed to auto save history, ignoring"));
            }
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self {
            version: Some(crate_version!().into()),
            files: Vec::new(),
            changed: false,
            autosave: None,
        }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// An error occurred while loading the history from a file.
    #[fail(display = "failed to load history from file")]
    Load(#[cause] LoadError),

    /// An error occurred while saving the history to a file.
    #[fail(display = "failed to save history to file")]
    Save(#[cause] SaveError),
}

impl From<LoadError> for Error {
    fn from(err: LoadError) -> Self {
        Error::Load(err)
    }
}

impl From<SaveError> for Error {
    fn from(err: SaveError) -> Self {
        Error::Save(err)
    }
}

#[derive(Debug, Fail)]
pub enum LoadError {
    /// Failed to read the file contents from the given file.
    #[fail(display = "failed to read from the history file")]
    Read(#[cause] IoError),

    /// Failed to parse the loaded file.
    #[fail(display = "failed to parse the file contents")]
    Parse(#[cause] DeError),
}

impl From<IoError> for LoadError {
    fn from(err: IoError) -> Self {
        LoadError::Read(err)
    }
}

impl From<DeError> for LoadError {
    fn from(err: DeError) -> Self {
        LoadError::Parse(err)
    }
}

#[derive(Debug, Fail)]
pub enum SaveError {
    /// No autosave file path was present, failed to save.
    #[fail(display = "no autosave file path specified")]
    NoPath,

    /// Failed to serialize the history for saving.
    #[fail(display = "failed to serialize the history for saving")]
    Serialize(#[cause] SerError),

    /// Failed to write to the history file.
    #[fail(display = "failed to write to the history file")]
    Write(#[cause] IoError),

    /// Failed to set file permissions to the history file.
    #[fail(display = "failed to set permissions to the history file")]
    SetPermissions(#[cause] IoError),

    /// Failed to delete the history file, which was tried because there
    /// are no history items to save.
    #[fail(display = "failed to delete history file, because history is empty")]
    Delete(#[cause] IoError),
}

impl From<SerError> for SaveError {
    fn from(err: SerError) -> Self {
        SaveError::Serialize(err)
    }
}

impl From<IoError> for SaveError {
    fn from(err: IoError) -> Self {
        SaveError::Write(err)
    }
}
