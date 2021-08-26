use std::env::current_dir;
use std::fs::create_dir_all;
#[cfg(feature = "archive")]
use std::io::Error as IoError;
use std::path::{self, PathBuf};
use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use failure::Fail;
use ffsend_api::action::download::{Download as ApiDownload, Error as DownloadError};
use ffsend_api::action::exists::{Error as ExistsError, Exists as ApiExists};
use ffsend_api::action::metadata::{Error as MetadataError, Metadata as ApiMetadata};
use ffsend_api::action::version::Error as VersionError;
use ffsend_api::file::remote_file::{FileParseError, RemoteFile};
use ffsend_api::pipe::ProgressReporter;
#[cfg(feature = "archive")]
use tempfile::{Builder as TempBuilder, NamedTempFile};

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

        // A temporary archive file, only used when archiving
        // The temporary file is stored here, to ensure it's lifetime exceeds the upload process
        #[cfg(feature = "archive")]
        let mut tmp_archive: Option<NamedTempFile> = None;

        // Check whether to extract
        #[cfg(feature = "archive")]
        let mut extract = matcher_download.extract();

        #[cfg(feature = "archive")]
        {
            // Ask to extract if downloading an archive
            if !extract && metadata.metadata().is_archive() {
                if prompt_yes(
                    "You're downloading an archive, extract it into the selected directory?",
                    Some(true),
                    &matcher_main,
                ) {
                    extract = true;
                }
            }
        }

        // Prepare the download target and output path to use
        #[cfg(feature = "archive")]
        let output_dir = !extract;
        #[cfg(not(feature = "archive"))]
        let output_dir = false;
        #[allow(unused_mut)]
        let mut target = Self::prepare_path(
            &target,
            metadata.metadata().name(),
            &matcher_main,
            output_dir,
        );
        #[cfg(feature = "archive")]
        let output_path = target.clone();

        #[cfg(feature = "archive")]
        {
            // Allocate an archive file, and update the download and target paths
            if extract {
                // TODO: select the extension dynamically
                let archive_extention = ".tar";

                // Allocate a temporary file to download the archive to
                tmp_archive = Some(
                    TempBuilder::new()
                        .prefix(&format!(".{}-archive-", crate_name!()))
                        .suffix(archive_extention)
                        .tempfile()
                        .map_err(ExtractError::TempFile)?,
                );
                if let Some(tmp_archive) = &tmp_archive {
                    target = tmp_archive.path().to_path_buf();
                }
            }
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

        // Extract the downloaded file if working with an archive
        #[cfg(feature = "archive")]
        {
            if extract {
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
    /// If `file` is set to false, no file name is included and the path
    /// will point to a directory.
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
        file: bool,
    ) -> PathBuf {
        // Select the path to use
        let mut target = Self::select_path(&target, name_hint);

        // Use the parent directory, if we don't want a file
        if !file {
            target = target.parent().unwrap().to_path_buf();
        }

        // Ask to overwrite
        if file && target.exists() && !main_matcher.force() {
            eprintln!(
                "The path '{}' already exists",
                target.to_str().unwrap_or("?"),
            );
            if !prompt_yes("Overwrite?", None, main_matcher) {
                println!("Download cancelled");
                quit();
            }
        }

        {
            // Get the deepest directory, as we have to ensure it exists
            let dir = if file {
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
