use std::env::current_dir;
use std::fs;
use std::io::{Error as IoError, Write};
use std::path::Path;
#[cfg(feature = "archive")]
use std::path::PathBuf;
#[cfg(feature = "archive")]
use std::process::exit;
use std::sync::{Arc, Mutex};

use clap::ArgMatches;
use failure::Fail;
use ffsend_api::action::params::ParamsDataBuilder;
use ffsend_api::action::upload::{Error as UploadError, Upload as ApiUpload};
use ffsend_api::action::version::Error as VersionError;
use ffsend_api::config::{upload_size_max, UPLOAD_SIZE_MAX_RECOMMENDED};
use ffsend_api::pipe::ProgressReporter;
use pathdiff::diff_paths;
use prettytable::{format::FormatBuilder, Cell, Row, Table};
#[cfg(feature = "qrcode")]
use qr2term::print_qr;
use tempfile::{Builder as TempBuilder, NamedTempFile};

use super::select_api_version;
#[cfg(feature = "archive")]
use crate::archive::archiver::Archiver;
use crate::client::create_config;
use crate::cmd::matcher::{MainMatcher, Matcher, UploadMatcher};
#[cfg(feature = "history")]
use crate::history_tool;
use crate::progress::ProgressBar;
#[cfg(feature = "urlshorten")]
use crate::urlshorten;
#[cfg(feature = "clipboard")]
use crate::util::set_clipboard;
use crate::util::{
    format_bytes, open_url, print_error, print_error_msg, prompt_yes, quit, quit_error_msg,
    rand_alphanum_string, stdin_read_file, ErrorHintsBuilder, StdinErr,
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

        // The file name to use
        #[allow(unused_mut)]
        let mut file_name = matcher_upload.name().map(|s| s.to_owned());

        // The selected files
        let mut files = matcher_upload.files();

        // If file is `-`, upload from stdin
        // TODO: write stdin directly to file, or directly to upload buffer
        let mut tmp_stdin: Option<NamedTempFile> = None;
        if files.len() == 1 && files[0] == "-" {
            // Obtain data from stdin
            let data = stdin_read_file(!matcher_main.quiet()).map_err(Error::Stdin)?;

            // Create temporary stdin buffer file
            tmp_stdin = Some(
                TempBuilder::new()
                    .prefix(&format!(".{}-stdin-", crate_name!()))
                    .tempfile()
                    .map_err(Error::StdinTempFile)?,
            );
            let file = tmp_stdin.as_ref().unwrap();

            // Fill temporary file with data, update list of files we upload, suggest name
            file.as_file()
                .write_all(&data)
                .map_err(Error::StdinTempFile)?;
            files = vec![file
                .path()
                .to_str()
                .expect("failed to obtain file name for stdin buffer file")];
            file_name = file_name.or_else(|| Some("stdin.txt".into()));
        }

        // Get API parameters
        #[allow(unused_mut)]
        let mut paths: Vec<_> = files
            .into_iter()
            .map(|p| Path::new(p).to_path_buf())
            .collect();
        let mut path = Path::new(paths.first().unwrap()).to_path_buf();
        let host = matcher_upload.host();

        // All paths must exist
        // TODO: ensure the file exists and is accessible
        for path in &paths {
            if !path.exists() {
                quit_error_msg(
                    format!("the path '{}' does not exist", path.to_str().unwrap_or("?")),
                    ErrorHintsBuilder::default().build().unwrap(),
                );
            }
        }

        // A temporary archive file, only used when archiving
        // The temporary file is stored here, to ensure it's lifetime exceeds the upload process
        #[allow(unused_mut)]
        #[cfg(feature = "archive")]
        let mut tmp_archive: Option<NamedTempFile> = None;

        #[cfg(feature = "archive")]
        {
            // Determine whether to archive, we must archive for multiple files/directory
            let mut archive = matcher_upload.archive();
            if !archive {
                if paths.len() > 1 {
                    if prompt_yes(
                        "You've selected multiple files, only a single file may be uploaded.\n\
                         Archive the files into a single file?",
                        Some(true),
                        &matcher_main,
                    ) {
                        archive = true;
                    } else {
                        exit(1);
                    }
                } else if path.is_dir() {
                    if prompt_yes(
                        "You've selected a directory, only a single file may be uploaded.\n\
                         Archive the directory into a single file?",
                        Some(true),
                        &matcher_main,
                    ) {
                        archive = true;
                    } else {
                        exit(1);
                    }
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
                        // Derive name from given file
                        if paths.len() == 1 {
                            file_name = Some(
                                path.canonicalize()
                                    .map_err(|err| ArchiveError::FileName(Some(err)))?
                                    .file_name()
                                    .ok_or(ArchiveError::FileName(None))?
                                    .to_str()
                                    .map(|s| s.to_owned())
                                    .ok_or(ArchiveError::FileName(None))?,
                            );
                        } else {
                            // Unable to derive file name from paths, generate random
                            file_name = Some(format!("ffsend-archive-{}", rand_alphanum_string(8)));
                        }
                    }

                    // Get the current working directory, including working directory as highest possible root, canonicalize it
                    let working_dir =
                        current_dir().expect("failed to get current working directory");
                    let shared_dir = {
                        let mut paths = paths.clone();
                        paths.push(working_dir.clone());
                        match shared_dir(paths) {
                            Some(p) => p,
                            None => quit_error_msg(
                                "when archiving, all files must be within a same directory",
                                ErrorHintsBuilder::default().verbose(false).build().unwrap(),
                            ),
                        }
                    };

                    // Build an archiver, append each file
                    let mut archiver = Archiver::new(archive_file);
                    for path in &paths {
                        // Canonicalize the path
                        let mut path = Path::new(path).to_path_buf();
                        if let Ok(p) = path.canonicalize() {
                            path = p;
                        }

                        // Find relative name to share dir, used to derive name from
                        let name = diff_paths(&path, &shared_dir)
                            .expect("failed to determine relative path of file to archive");
                        let name = name.to_str().expect("failed to get file path");

                        // Add file to archiver
                        archiver
                            .append_path(name, &path)
                            .map_err(ArchiveError::AddFile)?;
                    }

                    // Finish the archival process, writes the archive file
                    archiver.finish().map_err(ArchiveError::Write)?;

                    // Append archive extension to name, set to upload archived file
                    if let Some(ref mut file_name) = file_name {
                        file_name.push_str(archive_extention);
                    }
                    path = archive_path;
                    paths.clear();
                }
            }
        }

        // Quit with error when uploading multiple files or directory, if we cannot archive
        #[cfg(not(feature = "archive"))]
        {
            if paths.len() > 1 {
                quit_error_msg(
                    "uploading multiple files is not supported, ffsend must be compiled with 'archive' feature for this",
                    ErrorHintsBuilder::default()
                        .verbose(false)
                        .build()
                        .unwrap(),
                );
            }
            if path.is_dir() {
                quit_error_msg(
                    "uploading a directory is not supported, ffsend must be compiled with 'archive' feature for this",
                    ErrorHintsBuilder::default()
                        .verbose(false)
                        .build()
                        .unwrap(),
                );
            }
        }

        // Create a reqwest client capable for uploading files
        let client_config = create_config(&matcher_main);
        let client = client_config.clone().client(false);

        // Determine the API version to use
        let mut desired_version = matcher_main.api();
        select_api_version(&client, host.clone(), &mut desired_version)?;
        let api_version = desired_version.version().unwrap();

        // We do not authenticate for now
        let auth = false;

        // TODO: extract this into external function
        {
            // Determine the max file size
            // TODO: set false parameter to authentication state
            let max_size = upload_size_max(api_version, auth);

            // Get the file size, fail on empty files, warn about large files
            if let Ok(size) = path.metadata().map(|m| m.len()) {
                // Enforce files not being 0 bytes
                if size == 0 && !matcher_main.force() {
                    quit_error_msg(
                        "uploading a file with a size of 0 bytes is not supported",
                        ErrorHintsBuilder::default()
                            .force(true)
                            .verbose(false)
                            .build()
                            .unwrap(),
                    )
                }

                // Enforce maximum file size
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
                }

                // Enforce maximum recommended size
                if size > UPLOAD_SIZE_MAX_RECOMMENDED && !matcher_main.force() {
                    // The file is larger than the recommended maximum, warn
                    eprintln!(
                        "The file size is {}, bigger than the recommended maximum of {}",
                        format_bytes(size),
                        format_bytes(UPLOAD_SIZE_MAX_RECOMMENDED),
                    );

                    // Prompt the user to continue, quit if the user answered no
                    if !prompt_yes("Continue uploading?", Some(true), &matcher_main) {
                        eprintln!("Upload cancelled");
                        quit();
                    }
                }
            } else {
                print_error_msg("failed to check the file size, ignoring");
            }
        }

        // TODO: assert max expiry time for file

        // Create a reqwest client capable for uploading files
        let transfer_client = client_config.client(true);

        // Create a progress bar reporter
        let progress_bar = Arc::new(Mutex::new(ProgressBar::new_upload()));

        // Build a parameters object to set for the file
        let params = {
            // Build the parameters data object
            let params = ParamsDataBuilder::default()
                .download_limit(
                    matcher_upload
                        .download_limit(&matcher_main, api_version, auth)
                        .map(|d| d as u8),
                )
                .expiry_time(matcher_upload.expiry_time(&matcher_main, api_version, auth))
                .build()
                .unwrap();

            // Wrap the data in an option if not empty
            if params.is_empty() {
                None
            } else {
                Some(params)
            }
        };

        // Build the progress reporter
        let progress_reporter: Arc<Mutex<dyn ProgressReporter>> = progress_bar;

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
        #[allow(unused_mut)]
        let mut url = file.download_url(true);

        // Shorten the share URL if requested, prompt the user to confirm
        #[cfg(feature = "urlshorten")]
        {
            if matcher_upload.shorten() {
                if prompt_yes("URL shortening is a security risk. This shares the secret URL with a 3rd party.\nDo you want to shorten the share URL?", Some(false), &matcher_main) {
                    match urlshorten::shorten_url(&client, &url) {
                        Ok(short) => url = short,
                        Err(err) => print_error(
                            err.context("failed to shorten share URL, ignoring")
                                .compat(),
                        ),
                    }
                }
            }
        }

        // Report the result
        if !matcher_main.quiet() {
            // Create a table
            let mut table = Table::new();
            table.set_format(FormatBuilder::new().padding(0, 2).build());

            // Show the original URL when shortening, verbose and different
            #[cfg(feature = "urlshorten")]
            {
                let full_url = file.download_url(true);
                if matcher_main.verbose() && matcher_upload.shorten() && url != full_url {
                    table.add_row(Row::new(vec![
                        Cell::new("Full share link:"),
                        Cell::new(full_url.as_str()),
                    ]));
                }
            }

            if matcher_main.verbose() {
                // Show the share URL
                table.add_row(Row::new(vec![
                    Cell::new("Share link:"),
                    Cell::new(url.as_str()),
                ]));

                // Show a generate passphrase
                if password_generated {
                    table.add_row(Row::new(vec![
                        Cell::new("Passphrase:"),
                        Cell::new(&password.unwrap_or("?".into())),
                    ]));
                }

                // Show the owner token
                table.add_row(Row::new(vec![
                    Cell::new("Owner token:"),
                    Cell::new(file.owner_token().unwrap()),
                ]));
            } else {
                table.add_row(Row::new(vec![Cell::new(url.as_str())]));

                // Show a generate passphrase
                if password_generated {
                    table.add_row(Row::new(vec![Cell::new(&password.unwrap_or("?".into()))]));
                }
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

        // Copy the URL or command to the user's clipboard
        #[cfg(feature = "clipboard")]
        {
            if let Some(copy_mode) = matcher_upload.copy() {
                if let Err(err) = set_clipboard(copy_mode.build(url.as_str())) {
                    print_error(
                        err.context("failed to copy the share link to the clipboard, ignoring"),
                    );
                }
            }
        }

        // Print a QR code for the share URL
        #[cfg(feature = "qrcode")]
        {
            if matcher_upload.qrcode() {
                if let Err(err) = print_qr(url.as_str()) {
                    print_error(err.context("failed to print QR code, ignoring").compat());
                }
            }
        }

        // Close the temporary stdin buffer file, to ensure it's removed
        if let Some(tmp_stdin) = tmp_stdin.take() {
            if let Err(err) = tmp_stdin.close() {
                print_error(
                    err.context("failed to clean up temporary stdin buffer file, ignoring")
                        .compat(),
                );
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

        // Delete local files after uploading
        if matcher_upload.delete() {
            for path in &paths {
                if path.is_file() {
                    if let Err(err) = fs::remove_file(path) {
                        print_error(
                            Error::Delete(err)
                                .context("failed to delete local file after upload, ignoring")
                                .compat(),
                        );
                    }
                } else {
                    if let Err(err) = fs::remove_dir_all(path) {
                        print_error(
                            Error::Delete(err)
                                .context("failed to delete local directory after upload, ignoring")
                                .compat(),
                        );
                    }
                }
            }
        }

        Ok(())
    }
}

/// Find the deepest directory all given paths share.
///
/// This function canonicalizes the paths, make sure the paths exist.
///
/// Returns `None` if paths are using a different root.
///
/// # Examples
///
/// If the following paths are given:
///
/// - `/home/user/git/ffsend/src`
/// - `/home/user/git/ffsend/src/main.rs`
/// - `/home/user/git/ffsend/Cargo.toml`
///
/// The following is returned:
///
/// `/home/user/git/ffsend`
#[cfg(feature = "archive")]
fn shared_dir(paths: Vec<PathBuf>) -> Option<PathBuf> {
    // Any path must be given
    if paths.is_empty() {
        return None;
    }

    // Build vector
    let c: Vec<Vec<PathBuf>> = paths
        .into_iter()
        .map(|p| p.canonicalize().expect("failed to canonicalize path"))
        .map(|mut p| {
            // Start with parent if current path is file
            if p.is_file() {
                p = match p.parent() {
                    Some(p) => p.to_path_buf(),
                    None => return vec![],
                };
            }

            // Build list of path buffers for each path component
            let mut items = vec![p];
            #[allow(mutable_borrow_reservation_conflict)]
            while let Some(item) = items.last().unwrap().parent() {
                items.push(item.to_path_buf());
            }

            // Reverse as we built it in the wrong order
            items.reverse();
            items
        })
        .collect();

    // Find the index at which the paths are last shared at by walking through indices
    let i = (0..)
        .take_while(|i| {
            // Get path for first item, stop if none
            let base = &c[0].get(*i);
            if base.is_none() {
                return false;
            };

            // All other paths must equal at this index
            c.iter().skip(1).all(|p| &p.get(*i) == base)
        })
        .last();

    // Find the shared path
    i.map(|i| c[0][i].to_path_buf())
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

    /// An error occurred while deleting a local file after upload.
    #[fail(display = "failed to delete local file")]
    Delete(#[cause] IoError),

    /// An error occurred while reading data from stdin.
    #[fail(display = "failed to read data from stdin")]
    Stdin(#[cause] StdinErr),

    /// An error occurred while creating the temporary stdin file.
    #[fail(display = "failed to create temporary stdin buffer file")]
    StdinTempFile(#[cause] IoError),
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
