// TODO: remove all expect unwraps, replace them with proper errors

extern crate tempfile;

use std::fs::File;
use std::io::Error as IoError;
use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use failure::Fail;
use ffsend_api::action::params::ParamsDataBuilder;
use ffsend_api::action::upload::{
    Error as UploadError,
    Upload as ApiUpload,
};
use ffsend_api::config::{UPLOAD_SIZE_MAX, UPLOAD_SIZE_MAX_RECOMMENDED};
use ffsend_api::reader::ProgressReporter;
use ffsend_api::reqwest::Client;
use self::tempfile::{
    Builder as TempBuilder,
    NamedTempFile,
};

use archive::archiver::Archiver;
use cmd::matcher::{Matcher, MainMatcher, UploadMatcher};
#[cfg(feature = "history")]
use history_tool;
use progress::ProgressBar;
use util::{
    ErrorHintsBuilder,
    format_bytes,
    open_url,
    print_error,
    print_error_msg,
    prompt_yes,
    quit,
    quit_error_msg,
};
#[cfg(feature = "clipboard")]
use util::set_clipboard;

/// A file upload action.
pub struct Upload<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Upload<'a> {
    /// Construct a new upload action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self {
            cmd_matches,
        }
    }

    /// Invoke the upload action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), Error> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_upload = UploadMatcher::with(self.cmd_matches).unwrap();

        // Get API parameters
        let mut path = Path::new(matcher_upload.file()).to_path_buf();
        let host = matcher_upload.host();

        // TODO: ensure the file exists and is accessible

        // Get the file size to warn about large files
        if let Ok(size) = File::open(&path)
            .and_then(|f| f.metadata())
            .map(|m| m.len())
        {
            if size > UPLOAD_SIZE_MAX && !matcher_main.force() {
                // The file is too large, show an error and quit
                quit_error_msg(
                    format!(
                        "the file size is {}, bigger than the maximum allowed of {}",
                        format_bytes(size),
                        format_bytes(UPLOAD_SIZE_MAX),
                    ),
                    ErrorHintsBuilder::default()
                        .force(true)
                        .verbose(false)
                        .build()
                        .unwrap(),
                );
            } else if size > UPLOAD_SIZE_MAX_RECOMMENDED && !matcher_main.force() {
                // The file is larger than the recommended maximum, warn
                eprintln!(
                    "the file size is {}, bigger than the recommended maximum of {}",
                    format_bytes(size),
                    format_bytes(UPLOAD_SIZE_MAX_RECOMMENDED),
                );

                // Prompt the user to continue, quit if the user answered no
                if !prompt_yes("Continue uploading?", Some(true), &matcher_main) {
                    println!("Upload cancelled");
                    quit();
                }
            }
        } else {
            print_error_msg("failed to check the file size, ignoring");
        }

        // Create a reqwest client
        let client = Client::new();

        // Create a progress bar reporter
        let progress_bar = Arc::new(Mutex::new(ProgressBar::new_upload()));

        // Build a parameters object to set for the file
        let params = {
            // Build the parameters data object
            let mut params = ParamsDataBuilder::default()
                .download_limit(matcher_upload.download_limit())
                .build()
                .unwrap();

            // Wrap the data in an option if not empty
            if params.is_empty() {
                None
            } else {
                Some(params)
            }
        };

        // The file name to use
        let mut file_name = matcher_upload.name().map(|s| s.to_owned());

        // A temporary archive file, only used when archiving
        // The temporary file is stored here, to ensure it's lifetime exceeds the upload process
        let mut tmp_archive: Option<NamedTempFile> = None;
        let archive_extention = ".tar";

        // Archive the file if specified
        if matcher_upload.archive() {
            println!("Archiving file...");

            // Create a new temporary file to write the archive to
            tmp_archive = Some(
                TempBuilder::new()
                    .prefix(&format!(".{}-archive-", crate_name!()))
                    .suffix(archive_extention)
                    .tempfile()
                    .map_err(ArchiveError::TempFile)?
            );
            if let Some(tmp_archive) = &tmp_archive {
                // Get the path, and the actual file
                let archive_path = tmp_archive.path().to_path_buf();
                let archive_file = tmp_archive.as_file()
                    .try_clone()
                    .map_err(ArchiveError::CloneHandle)?;

                // Select the file name to use if not set
                if file_name.is_none() {
                    // TODO: use canonical path here
                    file_name = Some(
                        path.file_name()
                            .ok_or(ArchiveError::FileName)?
                            .to_str()
                            .map(|s| s.to_owned())
                            .expect("failed to create string from file name")
                    );
                }

                // Build an archiver and append the file
                let mut archiver = Archiver::new(archive_file);
                archiver.append_path(file_name.as_ref().unwrap(), &path)
                    .map_err(ArchiveError::AddFile)?;

                // Finish the archival process, writes the archive file
                archiver.finish().map_err(ArchiveError::Write)?;

                // Append archive extention to name, set to upload archived file
                if let Some(ref mut file_name) = file_name {
                    file_name.push_str(archive_extention);
                }
                path = archive_path;
            }
        }

        // Build the progress reporter
        let progress_reporter: Arc<Mutex<ProgressReporter>> = progress_bar;

        // Execute an upload action
        let file = ApiUpload::new(
            host,
            path.clone(),
            file_name,
            matcher_upload.password(),
            params,
        ).invoke(&client, &progress_reporter)?;

        // Get the download URL, and report it in the console
        let url = file.download_url(true);
        println!("Download URL: {}", url);
        println!("Owner token: {}", file.owner_token().unwrap());

        // Add the file to the history manager
        #[cfg(feature = "history")]
        history_tool::add(&matcher_main, file.clone(), false);

        // Open the URL in the browser
        if matcher_upload.open() {
            if let Err(err) = open_url(&url) {
                print_error(
                    err.context("failed to open the URL in the browser")
                );
            };
        }

        // Copy the URL in the user's clipboard
        #[cfg(feature = "clipboard")]
        {
            if matcher_upload.copy() && set_clipboard(url.as_str().to_owned()).is_err() {
                print_error_msg("failed to copy the URL to the clipboard");
            }
        }

        // Close the temporary zip file, to ensure it's removed
        if let Some(tmp_archive) = tmp_archive.take() {
            if let Err(err) = tmp_archive.close() {
                print_error(
                    err.context("failed to clean up temporary archive file, ignoring").compat(),
                );
            }
        }

        Ok(())
    }
}

#[derive(Debug, Fail)]
pub enum Error {
    /// An error occurred while archiving the file to upload.
    #[fail(display = "failed to archive file to upload")]
    Archive(#[cause] ArchiveError),

    /// An error occurred while uploading the file.
    #[fail(display = "")]
    Upload(#[cause] UploadError),
}

impl From<ArchiveError> for Error {
    fn from(err: ArchiveError) -> Error {
        Error::Archive(err)
    }
}

impl From<UploadError> for Error {
    fn from(err: UploadError) -> Error {
        Error::Upload(err)
    }
}

#[derive(Debug, Fail)]
pub enum ArchiveError {
    /// An error occurred while creating the temporary archive file.
    #[fail(display = "failed to create temporary archive file")]
    TempFile(#[cause] IoError),

    /// An error occurred while internally cloning the handle to the temporary archive file.
    #[fail(display = "failed to clone handle to temporary archive file")]
    CloneHandle(#[cause] IoError),

    /// Failed to infer a file name for the archive.
    #[fail(display = "failed to infer a file name for the archive")]
    FileName,

    /// Failed to add a file or directory to the archive.
    #[fail(display = "failed to add file to the archive")]
    AddFile(#[cause] IoError),

    /// Failed to write the created archive to the disk.
    #[fail(display = "failed to write archive to disk")]
    Write(#[cause] IoError),
}
