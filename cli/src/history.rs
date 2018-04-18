extern crate toml;

use std::fs;
use std::io::Read;
use std::path::PathBuf;

use ffsend_api::file::remote_file::RemoteFile;

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
    /// If the file doesn't exist, create a new empty history instance.
    ///
    /// Autosaving will be enabled, and will save to the given file path.
    pub fn load_or_new(file: PathBuf) -> Result<Self, ()> {
        if file.is_file() {
            Self::load(file)
        } else {
            Ok(Self::new(Some(file)))
        }
    }

    /// Load the history from the given file.
    pub fn load(path: PathBuf) -> Result<Self, ()> {
        // Read the file to a string
        // TODO: handle error
        let data = fs::read_to_string(path.clone()).unwrap();

        // Parse the data, set the autosave path
        let mut history: Self = toml::from_str(&data).unwrap();
        history.autosave = Some(path);

        Ok(history)
    }

    /// Save the history to the internal autosave file.
    pub fn save(&mut self) -> Result<(), ()> {
        // Build the data
        // TODO: handle error
        let data = toml::to_string(self).unwrap();

        // Write to a file
        // TODO: handle error
        fs::write(self.autosave.as_ref().unwrap(), data).unwrap();

        // There are no new changes, set the flag
        self.changed = false;

        Ok(())
    }

    /// Add the given remote file to the history.
    pub fn add(&mut self, file: RemoteFile) {
        self.files.push(file);
        self.changed = true;
    }

    /// Get all files.
    pub fn files(&self) -> &Vec<RemoteFile> {
        &self.files
    }
}

impl Drop for History {
    fn drop(&mut self) {
        // Automatically save if enabled and something was changed
        if self.autosave.is_some() && self.changed {
            self.save();
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
