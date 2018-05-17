use std::env::current_dir;
use std::fs::create_dir_all;
use std::path::{self, PathBuf};
use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use failure::Fail;
use ffsend_api::action::download::{
    Download as ApiDownload,
    Error as DownloadError,
};
use ffsend_api::action::exists::{
    Error as ExistsError,
    Exists as ApiExists,
};
use ffsend_api::action::metadata::{
    Error as MetadataError,
    Metadata as ApiMetadata,
};
use ffsend_api::file::remote_file::{FileParseError, RemoteFile};
use ffsend_api::reader::ProgressReporter;
use ffsend_api::reqwest::Client;

use cmd::matcher::{
    Matcher,
    download::DownloadMatcher,
    main::MainMatcher,
};
#[cfg(feature = "history")]
use history_tool;
use progress::ProgressBar;
use util::{
    ensure_enough_space,
    ensure_password,
    ErrorHints,
    prompt_yes,
    quit,
    quit_error,
    quit_error_msg,
};

/// A file download action.
pub struct Download<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Download<'a> {
    /// Construct a new download action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the download action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), Error> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_download = DownloadMatcher::with(self.cmd_matches).unwrap();

        // Get the share URL
        let url = matcher_download.url();

        // Create a reqwest client
        let client = Client::new();

        // Parse the remote file based on the share URL
        let file = RemoteFile::parse_url(url, None)?;

        // Get the target file or directory, and the password
        let target = matcher_download.output();
        let mut password = matcher_download.password();

        // Check whether the file exists
        let exists = ApiExists::new(&file).invoke(&client)?;
        if !exists.exists() {
            // Remove the file from the history manager if it does not exist
            #[cfg(feature = "history")]
            history_tool::remove(&matcher_main, &file);

            return Err(Error::Expired);
        }

        // Ensure a password is set when required
        ensure_password(&mut password, exists.has_password(), &matcher_main);

        // Fetch the file metadata
        let metadata = ApiMetadata::new(
            &file,
            password.clone(),
            false,
        ).invoke(&client)?;

        // Prepare the output path to use
        let target = Self::prepare_path(
            &target,
            metadata.metadata().name(),
            &matcher_main,
        );

        // Ensure there is enough disk space available when not being forced
        if !matcher_main.force() {
            ensure_enough_space(target.parent().unwrap(), metadata.size());
        }

        // Create a progress bar reporter
        let progress_bar = Arc::new(Mutex::new(ProgressBar::new_download()));
        let progress_reader: Arc<Mutex<ProgressReporter>> = progress_bar;

        // Execute an download action
        ApiDownload::new(
            &file,
            target,
            password,
            false,
            Some(metadata),
        ).invoke(&client, &progress_reader)?;

        // Add the file to the history
        #[cfg(feature = "history")]
        history_tool::add(&matcher_main, file, true);

        // TODO: open the file, or it's location
        // TODO: copy the file location

        Ok(())
    }

    /// This methods prepares a full file path to use for the file to
    /// download, based on the current directory, the original file name,
    /// and the user input.
    ///
    /// If no file name was given, the original file name is used.
    ///
    /// The full path including the name is returned.
    ///
    /// This method will check whether a file is overwitten, and whether
    /// parent directories must be created.
    ///
    /// The program will quit with an error message if a problem occurs.
    fn prepare_path(
        target: &PathBuf,
        name_hint: &str,
        main_matcher: &MainMatcher,
    ) -> PathBuf {
        // Select the path to use
        let target = Self::select_path(&target, name_hint);

        // Ask to overwrite
        if target.exists() && !main_matcher.force() {
            eprintln!(
                "The path '{}' already exists",
                target.to_str().unwrap_or("?"),
            );
            if !prompt_yes("Overwrite?", None, main_matcher) {
                println!("Download cancelled");
                quit();
            }
        }

        // Validate the parent directory exists
        match target.parent() {
            Some(parent) => if !parent.is_dir() {
                // Prompt to create them if not forced
                if !main_matcher.force() {
                    eprintln!(
                        "The directory '{}' doesn't exists",
                        parent.to_str().unwrap_or("?"),
                    );
                    if !prompt_yes("Create it?", Some(true), main_matcher) {
                        println!("Download cancelled");
                        quit();
                    }
                }

                // Create the parent directories
                if let Err(err) = create_dir_all(parent) {
                    quit_error(err.context(
                        "failed to create parent directories for output file",
                    ), ErrorHints::default());
                }
            },
            None => quit_error_msg(
                "invalid output file path",
                ErrorHints::default(),
            ),
        }

        target
    }

    /// This methods prepares a full file path to use for the file to
    /// download, based on the current directory, the original file name,
    /// and the user input.
    ///
    /// If no file name was given, the original file name is used.
    ///
    /// The full path including the file name will be returned.
    fn select_path(target: &PathBuf, name_hint: &str) -> PathBuf {
        // If we're already working with a file, canonicalize and return
        if target.is_file() {
            match target.canonicalize() {
                Ok(target) => return target,
                Err(err) => quit_error(
                    err.context("failed to canonicalize target path"),
                    ErrorHints::default(),
                ),
            }
        }

        // Append the name hint if this is a directory, canonicalize and return
        if target.is_dir() {
            match target.canonicalize() {
                Ok(target) => return target.join(name_hint),
                Err(err) => quit_error(
                    err.context("failed to canonicalize target path"),
                    ErrorHints::default(),
                ),
            }
        }

        // TODO: canonicalize parent if it exists

        // Get the path string
        let path = target.to_str();

        // If the path is emtpy, use the working directory with the name hint
        let use_workdir = path
            .map(|path| path.trim().is_empty())
            .unwrap_or(true);
        if use_workdir {
            match current_dir() {
                Ok(target) => return target.join(name_hint),
                Err(err) => quit_error(err.context(
                    "failed to determine working directory to use for the output file"
                ), ErrorHints::default()),
            }
        }
        let path = path.unwrap();

        // Make the target mutable
        let mut target = target.clone();

        // If the path ends with a separator, append the name hint
        if path.trim().ends_with(path::is_separator) {
            target = target.join(name_hint);
        }

        // If relative, use the working directory as base
        if target.is_relative() {
            match current_dir() {
                Ok(workdir) => target = workdir.join(target),
                Err(err) => quit_error(err.context(
                        "failed to determine working directory to use for the output file"
                    ), ErrorHints::default()),
            }
        }

        target
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "invalid share URL")]
    InvalidUrl(#[cause] FileParseError),

    /// An error occurred while checking if the file exists.
    #[fail(display = "failed to check whether the file exists")]
    Exists(#[cause] ExistsError),

    /// An error occurred while fetching metadata.
    #[fail(display = "failed to fetch file metadata")]
    Metadata(#[cause] MetadataError),

    /// An error occurred while downloading the file.
    #[fail(display = "")]
    Download(#[cause] DownloadError),

    /// The given Send file has expired, or did never exist in the first place.
    #[fail(display = "the file has expired or did never exist")]
    Expired,
}

impl From<FileParseError> for Error {
    fn from(err: FileParseError) -> Error {
        Error::InvalidUrl(err)
    }
}

impl From<ExistsError> for Error {
    fn from(err: ExistsError) -> Error {
        Error::Exists(err)
    }
}

impl From<MetadataError> for Error {
    fn from(err: MetadataError) -> Error {
        Error::Metadata(err)
    }
}

impl From<DownloadError> for Error {
    fn from(err: DownloadError) -> Error {
        Error::Download(err)
    }
}
