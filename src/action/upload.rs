use std::fs::File;
#[cfg(feature = "archive")]
use std::io::Error as IoError;
use std::path::Path;
use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use failure::Fail;
use ffsend_api::action::params::ParamsDataBuilder;
use ffsend_api::action::upload::{Error as UploadError, Upload as ApiUpload};
use ffsend_api::action::version::Error as VersionError;
use ffsend_api::config::{upload_size_max, UPLOAD_SIZE_MAX_RECOMMENDED};
use ffsend_api::pipe::ProgressReporter;
use prettytable::{format::FormatBuilder, Cell, Row, Table};
#[cfg(feature = "archive")]
use tempfile::{Builder as TempBuilder, NamedTempFile};

use super::select_api_version;
#[cfg(feature = "archive")]
use crate::archive::archiver::Archiver;
use crate::client::create_transfer_client;
use crate::cmd::matcher::{MainMatcher, Matcher, UploadMatcher};
#[cfg(feature = "history")]
use crate::history_tool;
use crate::progress::ProgressBar;
#[cfg(feature = "clipboard")]
use crate::util::set_clipboard;
use crate::util::{
    format_bytes, open_url, print_error, print_error_msg, prompt_yes, quit, quit_error_msg,
    ErrorHintsBuilder,
};

/// A file upload action.
pub struct Upload<'a> {
    cmd_matches: &'a ArgMatches<'a>,
}

impl<'a> Upload<'a> {
    /// Construct a new upload action.
    pub fn new(cmd_matches: &'a ArgMatches<'a>) -> Self {
        Self { cmd_matches }
    }

    /// Invoke the upload action.
    // TODO: create a trait for this method
    pub fn invoke(&self) -> Result<(), Error> {
        // Create the command matchers
        let matcher_main = MainMatcher::with(self.cmd_matches).unwrap();
        let matcher_upload = UploadMatcher::with(self.cmd_matches).unwrap();

        // Get API parameters
        #[allow(unused_mut)]
        let mut path = Path::new(matcher_upload.file()).to_path_buf();
        let host = matcher_upload.host();

        // Create a reqwest client capable for uploading files
        let client = create_transfer_client(&matcher_main);

        // Determine the API version to use
        let mut desired_version = matcher_main.api();
        select_api_version(&client, host.clone(), &mut desired_version)?;
        let api_version = desired_version.version().unwrap();

        // TODO: ensure the file exists and is accessible

        // Determine the max file size
        // TODO: set false parameter to authentication state
        let max_size = upload_size_max(api_version, false);

        // Get the file size to warn about large files
        if let Ok(size) = File::open(&path)
            .and_then(|f| f.metadata())
            .map(|m| m.len())
        {
            if size > max_size && !matcher_main.force() {
                // The file is too large, show an error and quit
                quit_error_msg(
                    format!(
                        "the file size is {}, bigger than the maximum allowed of {}",
                        format_bytes(size),
                        format_bytes(max_size),
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
                    "The file size is {}, bigger than the recommended maximum of {}",
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

        // Create a reqwest client capable for uploading files
        let transfer_client = create_transfer_client(&matcher_main);

        // Create a progress bar reporter
        let progress_bar = Arc::new(Mutex::new(ProgressBar::new_upload()));

        // Build a parameters object to set for the file
        let params = {
            // Build the parameters data object
            let params = ParamsDataBuilder::default()
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
        #[allow(unused_mut)]
        let mut file_name = matcher_upload.name().map(|s| s.to_owned());

        // A temporary archive file, only used when archiving
        // The temporary file is stored here, to ensure it's lifetime exceeds the upload process
        #[allow(unused_mut)]
        #[cfg(feature = "archive")]
        let mut tmp_archive: Option<NamedTempFile> = None;

        #[cfg(feature = "archive")]
        {
            // Determine whether to archive, ask if a directory was selected
            let mut archive = matcher_upload.archive();
            if !archive && path.is_dir() {
                if prompt_yes(
                    "You've selected a directory, only a single file may be uploaded.\n\
                     Archive the directory into a single file?",
                    Some(true),
                    &matcher_main,
                ) {
                    archive = true;
                }
            }

            // Archive the selected file or directory
            if archive {
                eprintln!("Archiving...");
                let archive_extention = ".tar";

                // Create a new temporary file to write the archive to
                tmp_archive = Some(
                    TempBuilder::new()
                        .prefix(&format!(".{}-archive-", crate_name!()))
                        .suffix(archive_extention)
                        .tempfile()
                        .map_err(ArchiveError::TempFile)?,
                );
                if let Some(tmp_archive) = &tmp_archive {
                    // Get the path, and the actual file
                    let archive_path = tmp_archive.path().to_path_buf();
                    let archive_file = tmp_archive
                        .as_file()
                        .try_clone()
                        .map_err(ArchiveError::CloneHandle)?;

                    // Select the file name to use if not set
                    if file_name.is_none() {
                        file_name = Some(
                            path.canonicalize()
                                .map_err(|err| ArchiveError::FileName(Some(err)))?
                                .file_name()
                                .ok_or(ArchiveError::FileName(None))?
                                .to_str()
                                .map(|s| s.to_owned())
                                .ok_or(ArchiveError::FileName(None))?,
                        );
                    }

                    // Build an archiver and append the file
                    let mut archiver = Archiver::new(archive_file);
                    archiver
                        .append_path(file_name.as_ref().unwrap(), &path)
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
        }

        // Build the progress reporter
        let progress_reporter: Arc<Mutex<ProgressReporter>> = progress_bar;

        // Get the password to use and whether it was generated
        let password = matcher_upload.password();
        let (password, password_generated) =
            password.map(|(p, g)| (Some(p), g)).unwrap_or((None, false));

        // Execute an upload action, obtain the URL
        let reporter = if !matcher_main.quiet() {
            Some(&progress_reporter)
        } else {
            None
        };
        let file = ApiUpload::new(
            api_version,
            host,
            path.clone(),
            file_name,
            password.clone(),
            params,
        )
        .invoke(&transfer_client, reporter)?;
        let url = file.download_url(true);

        // Report the result
        if !matcher_main.quiet() {
            // Show a table
            let mut table = Table::new();
            table.set_format(FormatBuilder::new().padding(0, 2).build());
            table.add_row(Row::new(vec![
                Cell::new("Share link:"),
                Cell::new(url.as_str()),
            ]));
            if password_generated {
                table.add_row(Row::new(vec![
                    Cell::new("Passphrase:"),
                    Cell::new(&password.unwrap_or("?".into())),
                ]));
            }
            if matcher_main.verbose() {
                table.add_row(Row::new(vec![
                    Cell::new("Owner token:"),
                    Cell::new(file.owner_token().unwrap()),
                ]));
            }
            table.printstd();
        } else {
            println!("{}", url);
        }

        // Add the file to the history manager
        #[cfg(feature = "history")]
        history_tool::add(&matcher_main, file.clone(), false);

        // Open the URL in the browser
        if matcher_upload.open() {
            if let Err(err) = open_url(&url) {
                print_error(err.context("failed to open the share link in the browser"));
            };
        }

        // Copy the URL in the user's clipboard
        #[cfg(feature = "clipboard")]
        {
            if matcher_upload.copy() {
                if let Err(err) = set_clipboard(url.as_str().to_owned()) {
                    print_error(
                        err.context("failed to copy the share link to the clipboard, ignoring"),
                    );
                }
            }
        }

        #[cfg(feature = "archive")]
        {
            // Close the temporary zip file, to ensure it's removed
            if let Some(tmp_archive) = tmp_archive.take() {
                if let Err(err) = tmp_archive.close() {
                    print_error(
                        err.context("failed to clean up temporary archive file, ignoring")
                            .compat(),
                    );
                }
            }
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

    /// An error occurred while archiving the file to upload.
    #[cfg(feature = "archive")]
    #[fail(display = "failed to archive file to upload")]
    Archive(#[cause] ArchiveError),

    /// An error occurred while uploading the file.
    #[fail(display = "")]
    Upload(#[cause] UploadError),
}

impl From<VersionError> for Error {
    fn from(err: VersionError) -> Error {
        Error::Version(err)
    }
}

#[cfg(feature = "archive")]
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

#[cfg(feature = "archive")]
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
    FileName(Option<IoError>),

    /// Failed to add a file or directory to the archive.
    #[fail(display = "failed to add file to the archive")]
    AddFile(#[cause] IoError),

    /// Failed to write the created archive to the disk.
    #[fail(display = "failed to write archive to disk")]
    Write(#[cause] IoError),
}
