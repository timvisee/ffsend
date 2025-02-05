use std::env::current_dir;
use std::fs::{create_dir_all, File};
use std::io::Error as IoError;
use std::io::{self, BufReader, Read};
use std::path::{self, PathBuf};
use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use failure::Fail;
use ffsend_api::action::download::{Download as ApiDownload, Error as DownloadError};
use ffsend_api::action::exists::{Error as ExistsError, Exists as ApiExists};
use ffsend_api::action::metadata::{Error as MetadataError, Metadata as ApiMetadata};
use ffsend_api::action::version::Error as VersionError;
use ffsend_api::file::metadata::{ManifestFile, Metadata};
use ffsend_api::file::remote_file::{FileParseError, RemoteFile};
use ffsend_api::pipe::ProgressReporter;
use tempfile::Builder as TempBuilder;

use super::select_api_version;
#[cfg(feature = "archive")]
use crate::archive::archive::Archive;
use crate::client::create_config;
use crate::cmd::matcher::{download::DownloadMatcher, main::MainMatcher, Matcher};
#[cfg(feature = "history")]
use crate::history_tool;
use crate::progress::ProgressBar;
use crate::util::{
    ensure_enough_space, ensure_password, follow_url, print_error, prompt_yes, quit, quit_error,
    quit_error_msg, ErrorHints,
};

/// A file download action.
pub struct Download<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

/// Strategy after the download action
enum InvokeStrategy {
    /// Just download the single normal file
    Normal,
    /// Download multiple files at once
    Multi { files: Vec<ManifestFile> },
    /// Download a single archive and extract it
    #[cfg(feature = "archive")]
    Extract,
}

impl InvokeStrategy {
    /// Determine strategy from metadata and CLI input
    pub fn build(
        metadata: &Metadata,
        matcher_download: &DownloadMatcher,
        matcher_main: &MainMatcher,
    ) -> InvokeStrategy {
        // Check whether to extract
        #[cfg(feature = "archive")]
        {
            if matcher_download.extract() {
                return InvokeStrategy::Extract;
            }
            if metadata.is_archive() {
                if prompt_yes(
                    "You're downloading an archive, extract it into the selected directory?",
                    Some(true),
                    &matcher_main,
                ) {
                    return InvokeStrategy::Extract;
                }
            }
        }

        // Check whether multiple files or not
        if metadata.mime() == "send-archive" {
            if let Some(manifest) = metadata.manifest() {
                InvokeStrategy::Multi {
                    files: manifest.files().clone(),
                }
                // `files` will be used after the action, but `ApiDownload::new` consumes metadata.
                // Therefore, we should clone in advance.
            } else {
                quit_error_msg(
                    "invalid metadata for downloading multiple files, manifest unknown",
                    ErrorHints::default(),
                )
            }
        } else {
            InvokeStrategy::Normal
        }
    }

    /// Whether the strategy will finally output multiple files
    pub fn has_multi_outs(&self) -> bool {
        match self {
            InvokeStrategy::Normal => false,
            InvokeStrategy::Multi { .. } => true,
            #[cfg(feature = "archive")]
            InvokeStrategy::Extract => true,
        }
    }
}

impl<'a> Download<'a> {
    /// Construct a new download action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the download action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), Error> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_download = DownloadMatcher::with(self.cmd_matches).unwrap();

        // Create a regular client
        let client_config = create_config(&matcher_main);
        let client = client_config.clone().client(false);

        // Get the share URL, attempt to follow it
        let url = matcher_download.url();
        let url = match follow_url(&client, &url) {
            Ok(url) => url,
            Err(err) => {
                print_error(err.context("failed to follow share URL, ignoring").compat());
                url
            }
        };

        // Guess the host
        let host = matcher_download.guess_host(Some(url.clone()));

        // Determine the API version to use
        let mut desired_version = matcher_main.api();
        select_api_version(&client, host, &mut desired_version)?;
        let api_version = desired_version.version().unwrap();

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
        ensure_password(
            &mut password,
            exists.requires_password(),
            &matcher_main,
            false,
        );

        // Fetch the file metadata
        let metadata = ApiMetadata::new(&file, password.clone(), false).invoke(&client)?;

        // Determine strategy
        let strategy = InvokeStrategy::build(metadata.metadata(), &matcher_download, &matcher_main);

        // Prepare the download target and output path to use
        #[allow(unused_mut)]
        let mut target = Self::prepare_path(
            &target,
            metadata.metadata().name(),
            &matcher_main,
            &strategy,
        );
        #[cfg(feature = "archive")]
        let output_path = target.clone();

        // Allocate an archive file if there will be multiple outputs…
        let tmp_archive = match &strategy {
            InvokeStrategy::Normal => None,
            InvokeStrategy::Multi { .. } => Some(
                TempBuilder::new()
                    .prefix(&format!(".{}-archive-", crate_name!()))
                    .tempfile()
                    .map_err(SplitError::TempFile)?,
            ),
            #[cfg(feature = "archive")]
            InvokeStrategy::Extract => {
                // TODO: select the extension dynamically
                let archive_extension = ".tar";

                Some(
                    TempBuilder::new()
                        .prefix(&format!(".{}-archive-", crate_name!()))
                        .suffix(archive_extension)
                        .tempfile()
                        .map_err(ExtractError::TempFile)?,
                )
            }
        };
        // …and update the download and target paths
        if let Some(tmp_archive) = &tmp_archive {
            target = tmp_archive.path().to_path_buf();
        }

        // Ensure there is enough disk space available when not being forced
        if !matcher_main.force() {
            ensure_enough_space(target.parent().unwrap(), metadata.size());
        }

        // Create a progress bar reporter
        let progress_bar = Arc::new(Mutex::new(ProgressBar::new_download()));
        let progress_reader: Arc<Mutex<dyn ProgressReporter>> = progress_bar;

        // Create a transfer client
        let transfer_client = client_config.client(true);

        // Execute an download action
        let progress = if !matcher_main.quiet() {
            Some(progress_reader)
        } else {
            None
        };
        ApiDownload::new(api_version, &file, target, password, false, Some(metadata))
            .invoke(&transfer_client, progress)?;

        // Post process
        match strategy {
            InvokeStrategy::Multi { files } => {
                eprintln!("Splitting...");

                Self::split(tmp_archive.unwrap().into_file(), files).map_err(SplitError::Split)?;
            }
            InvokeStrategy::Normal => {
                // No need to post process
            }
            #[cfg(feature = "archive")]
            InvokeStrategy::Extract => {
                eprintln!("Extracting...");

                // Extract the downloaded file
                Archive::new(tmp_archive.unwrap().into_file())
                    .extract(output_path)
                    .map_err(ExtractError::Extract)?;
            }
        }

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
    /// If `strategy` will generate multiple outputs, no file name is included
    /// and the path will point to a directory.
    ///
    /// If no file name was given, the original file name is used.
    ///
    /// The full path including the name is returned.
    ///
    /// This method will check whether a file is overwritten, and whether
    /// parent directories must be created.
    ///
    /// The program will quit with an error message if a problem occurs.
    fn prepare_path(
        target: &PathBuf,
        name_hint: &str,
        main_matcher: &MainMatcher,
        strategy: &InvokeStrategy,
    ) -> PathBuf {
        // Select the path to use
        let mut target = Self::select_path(&target, name_hint);

        // Use the parent directory, if there will be multiple outputs
        if strategy.has_multi_outs() {
            target = target.parent().unwrap().to_path_buf();
        }

        // Ask to overwrite if any file already exists
        if !main_matcher.force() {
            match strategy {
                InvokeStrategy::Normal => {
                    if target.exists() {
                        eprintln!(
                            "The path '{}' already exists",
                            target.to_str().unwrap_or("?"),
                        );
                        if !prompt_yes("Overwrite?", None, main_matcher) {
                            println!("Download cancelled");
                            quit();
                        }
                    }
                }
                InvokeStrategy::Multi { files } => {
                    for ManifestFile { name, .. } in files {
                        let path = Self::select_path(&target, name);
                        if path.exists() {
                            eprintln!("The path '{}' already exists", path.to_str().unwrap_or("?"),);
                            if !prompt_yes("Overwrite?", None, main_matcher) {
                                println!("Download cancelled");
                                quit();
                            }
                        }
                    }
                }
                InvokeStrategy::Extract => {
                    // We can't determine what will be extracted now,
                    // so let it go.
                }
            }
        }

        {
            // Get the deepest directory, as we have to ensure it exists
            let dir = if !strategy.has_multi_outs() {
                match target.parent() {
                    Some(parent) => parent,
                    None => quit_error_msg("invalid output file path", ErrorHints::default()),
                }
            } else {
                &target
            };

            // Ensure the directory exists
            if !dir.is_dir() {
                // Prompt to create them if not forced
                if !main_matcher.force() {
                    eprintln!(
                        "The directory '{}' doesn't exists",
                        dir.to_str().unwrap_or("?"),
                    );
                    if !prompt_yes("Create it?", Some(true), main_matcher) {
                        println!("Download cancelled");
                        quit();
                    }
                }

                // Create the parent directories
                if let Err(err) = create_dir_all(dir) {
                    quit_error(
                        err.context("failed to create parent directories for output file"),
                        ErrorHints::default(),
                    );
                }
            }
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

        // If the path is empty, use the working directory with the name hint
        let use_workdir = path.map(|path| path.trim().is_empty()).unwrap_or(true);
        if use_workdir {
            match current_dir() {
                Ok(target) => return target.join(name_hint),
                Err(err) => quit_error(
                    err.context("failed to determine working directory to use for the output file"),
                    ErrorHints::default(),
                ),
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
                Err(err) => quit_error(
                    err.context("failed to determine working directory to use for the output file"),
                    ErrorHints::default(),
                ),
            }
        }

        target
    }

    /// Split the downloaded send-archive into multiple files
    fn split(archive: File, files: Vec<ManifestFile>) -> Result<(), IoError> {
        let mut reader = BufReader::new(archive);

        for ManifestFile { name, size, .. } in files {
            let mut writer = File::create(name)?;
            io::copy(&mut reader.by_ref().take(size), &mut writer)?;
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// Selecting the API version to use failed.
    // TODO: enable `api` hint!
    #[fail(display = "failed to select API version to use")]
    Version(#[cause] VersionError),

    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "invalid share link")]
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

    /// An error occurred while extracting the file.
    #[cfg(feature = "archive")]
    #[fail(display = "failed the extraction procedure")]
    Extract(#[cause] ExtractError),

    /// An error occurred while splitting send-archive into multiple files.
    #[fail(display = "failed the split procedure")]
    Split(#[cause] SplitError),

    /// The given Send file has expired, or did never exist in the first place.
    #[fail(display = "the file has expired or did never exist")]
    Expired,
}

impl From<VersionError> for Error {
    fn from(err: VersionError) -> Error {
        Error::Version(err)
    }
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

#[cfg(feature = "archive")]
impl From<ExtractError> for Error {
    fn from(err: ExtractError) -> Error {
        Error::Extract(err)
    }
}

impl From<SplitError> for Error {
    fn from(err: SplitError) -> Error {
        Error::Split(err)
    }
}

#[cfg(feature = "archive")]
#[derive(Debug, Fail)]
pub enum ExtractError {
    /// An error occurred while creating the temporary archive file.
    #[fail(display = "failed to create temporary archive file")]
    TempFile(#[cause] IoError),

    /// Failed to extract the file contents to the target directory.
    #[fail(display = "failed to extract archive contents to target directory")]
    Extract(#[cause] IoError),
}

#[derive(Debug, Fail)]
pub enum SplitError {
    /// An error occurred while creating the temporary file.
    #[fail(display = "failed to create temporary file")]
    TempFile(#[cause] IoError),

    /// Failed to split send-archive into multiple files.
    #[fail(display = "failed to split the file into multiple files")]
    Split(#[cause] IoError),
}
