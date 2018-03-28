use ffsend_api::action::download::Error as DownloadError;
use ffsend_api::action::upload::Error as UploadError;

#[derive(Fail, Debug)]
pub enum Error {
    /// An error occurred while invoking an action.
    #[fail(display = "")]
    Action(#[cause] ActionError),
}

#[derive(Debug, Fail)]
pub enum ActionError {
    /// An error occurred while invoking the upload action.
    // TODO: bind the upload cause here
    #[fail(display = "Failed to upload the specified file")]
    Upload(#[cause] UploadError),

    /// An error occurred while invoking the download action.
    #[fail(display = "Failed to download the requested file")]
    Download(#[cause] DownloadError),
}
