use ffsend_api::action::delete::Error as DeleteError;
use ffsend_api::action::exists::Error as ExistsError;
use ffsend_api::action::params::Error as ParamsError;
use ffsend_api::action::password::Error as PasswordError;
use ffsend_api::action::version::Error as VersionError;
use ffsend_api::file::remote_file::FileParseError;

use crate::action::download::Error as CliDownloadError;
use crate::action::generate::completions::Error as CliGenerateCompletionsError;
#[cfg(feature = "history")]
use crate::action::history::Error as CliHistoryError;
use crate::action::info::Error as CliInfoError;
use crate::action::upload::Error as CliUploadError;

#[derive(Fail, Debug)]
pub enum Error {
    /// An error occurred while invoking an action.
    #[fail(display = "")]
    Action(#[cause] ActionError),
}

impl From<CliDownloadError> for Error {
    fn from(err: CliDownloadError) -> Error {
        Error::Action(ActionError::Download(err))
    }
}

impl From<CliInfoError> for Error {
    fn from(err: CliInfoError) -> Error {
        Error::Action(ActionError::Info(err))
    }
}

impl From<CliUploadError> for Error {
    fn from(err: CliUploadError) -> Error {
        Error::Action(ActionError::Upload(err))
    }
}

impl From<ActionError> for Error {
    fn from(err: ActionError) -> Error {
        Error::Action(err)
    }
}

#[derive(Debug, Fail)]
pub enum ActionError {
    /// An error occurred while invoking the delete action.
    #[fail(display = "failed to delete the file")]
    Delete(#[cause] DeleteError),

    /// An error occurred while invoking the download action.
    #[fail(display = "failed to download the requested file")]
    Download(#[cause] CliDownloadError),

    /// An error occurred while invoking the exists action.
    #[fail(display = "failed to check whether the file exists")]
    Exists(#[cause] ExistsError),

    /// An error occurred while generating completions.
    #[fail(display = "failed to generate shell completions")]
    GenerateCompletions(#[cause] CliGenerateCompletionsError),

    /// An error occurred while processing the file history.
    #[cfg(feature = "history")]
    #[fail(display = "failed to process the history")]
    History(#[cause] CliHistoryError),

    /// An error occurred while invoking the info action.
    #[fail(display = "failed to fetch file info")]
    Info(#[cause] CliInfoError),

    /// An error occurred while invoking the params action.
    #[fail(display = "failed to change the parameters")]
    Params(#[cause] ParamsError),

    /// An error occurred while invoking the password action.
    #[fail(display = "failed to change the password")]
    Password(#[cause] PasswordError),

    /// An error occurred while invoking the version action.
    #[fail(display = "failed to determine server version")]
    Version(#[cause] VersionError),

    /// An error occurred while invoking the upload action.
    #[fail(display = "failed to upload the specified file")]
    Upload(#[cause] CliUploadError),

    /// Failed to parse a share URL, it was invalid.
    /// This error is not related to a specific action.
    #[fail(display = "invalid share URL")]
    InvalidUrl(#[cause] FileParseError),
}

impl From<DeleteError> for ActionError {
    fn from(err: DeleteError) -> ActionError {
        ActionError::Delete(err)
    }
}

impl From<ExistsError> for ActionError {
    fn from(err: ExistsError) -> ActionError {
        ActionError::Exists(err)
    }
}

impl From<CliGenerateCompletionsError> for ActionError {
    fn from(err: CliGenerateCompletionsError) -> ActionError {
        ActionError::GenerateCompletions(err)
    }
}

#[cfg(feature = "history")]
impl From<CliHistoryError> for ActionError {
    fn from(err: CliHistoryError) -> ActionError {
        ActionError::History(err)
    }
}

impl From<ParamsError> for ActionError {
    fn from(err: ParamsError) -> ActionError {
        ActionError::Params(err)
    }
}

impl From<PasswordError> for ActionError {
    fn from(err: PasswordError) -> ActionError {
        ActionError::Password(err)
    }
}

impl From<VersionError> for ActionError {
    fn from(err: VersionError) -> ActionError {
        ActionError::Version(err)
    }
}

impl From<FileParseError> for ActionError {
    fn from(err: FileParseError) -> ActionError {
        ActionError::InvalidUrl(err)
    }
}
