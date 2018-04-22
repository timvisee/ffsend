extern crate toml;

use std::fs;
use std::io::Error as IoError;
use std::path::PathBuf;

use failure::Fail;
use ffsend_api::file::remote_file::RemoteFile;
use self::toml::de::Error as DeError;
use self::toml::ser::Error as SerError;

use util::print_error;

#[derive(Serialize, Deserialize)]
pub struct History {
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
        use std::fs::File;
        use std::io::Read;
        let mut file = File::open(&path)?;
        let mut data = String::new();
        file.read_to_string(&mut data)?;

        // TODO: switch to this instead in stable Rust 1.26
        // let data = fs::read_to_string(&path)?;

        // Parse the data, set the autosave path
        let mut history: Self = toml::from_str(&data)?;
        history.autosave = Some(path);

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
        let path = self.autosave
            .as_ref()
            .ok_or(SaveError::NoPath)?;

        // If we have no files, remove the history file if it exists
        if self.files.is_empty() {
            if path.is_file() {
                fs::remove_file(&path)
                    .map_err(|err| SaveError::Delete(err))?;
            }
            return Ok(());
        }

        // Ensure the file parnet directories are available
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Build the data
        let data = toml::to_string(self)?;

        // Write to the file
        use std::fs::File;
        use std::io::Write;
        File::create(&path)?.write_all(data.as_ref())?;

        // TODO: switch to this instead in stable Rust 1.26
        // let data = fs::read_to_string(path.clone())?;

        // There are no new changes, set the flag
        self.changed = false;

        Ok(())
    }

    /// Load the history from the given path, add the given file, and save it
    /// again.
    /// If there is not history file at the given path, a new empty one will
    /// be created.
    pub fn load_add_save(path: PathBuf, file: RemoteFile) -> Result<(), Error> {
        let mut history = Self::load_or_new(path)?;
        history.add(file);
        history.save().map_err(|err| err.into())
    }

    /// Add the given remote file to the history.
    pub fn add(&mut self, file: RemoteFile) {
        self.files.push(file);
        self.changed = true;
    }

    /// Remove the given remote file, matched by it's file ID.
    ///
    /// If any file was removed, true is returned.
    pub fn remove(&mut self, file: &RemoteFile) -> bool {
        // Get the indices of files that have expired
        let expired_indices: Vec<usize> = self.files.iter()
            .enumerate()
            .filter(|(_, f)| f.id() == file.id())
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

    /// Get all files.
    pub fn files(&self) -> &Vec<RemoteFile> {
        &self.files
    }

    /// Garbage collect (remove) all files that have been expired,
    /// as defined by their `expire_at` property.
    ///
    /// If the expiry property is None (thus unknown), the file will be kept.
    ///
    /// The number of exired files is returned.
    pub fn gc(&mut self) -> usize {
        // Get a list of expired files
        let expired: Vec<RemoteFile> = self.files
            .iter()
            .filter(|f| f.has_expired(false))
            .cloned()
            .collect();

        // Remove the files
        for f in &expired {
            self.remove(f);
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
                print_error(
                    err.context("Failed to auto save history, ignoring"),
                );
            }
        }
    }
}

impl Default for History {
    fn default() -> Self {
        Self {
            files: Vec::new(),
            changed: false,
            autosave: None,
        }
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// An error occurred while loading the history from a file.
    #[fail(display = "Failed to load history from file")]
    Load(#[cause] LoadError),

    /// An error occurred while saving the history to a file.
    #[fail(display = "Failed to save history to file")]
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
    #[fail(display = "Failed to read from the history file")]
    Read(#[cause] IoError),

    /// Failed to parse the loaded file.
    #[fail(display = "Failed to parse the file contents")]
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
    #[fail(display = "No autosave file path specified")]
    NoPath,

    /// Failed to serialize the history for saving.
    #[fail(display = "Failed to serialize the history for saving")]
    Serialize(#[cause] SerError),

    /// Failed to write to the history file.
    #[fail(display = "Failed to write to the history file")]
    Write(#[cause] IoError),

    /// Failed to delete the history file, which was tried because there
    /// are no history items to save.
    #[fail(display = "Failed to delete history file, because history is empty")]
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
