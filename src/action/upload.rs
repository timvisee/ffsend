use super::super::send::file::File;

/// The response from the server after a file has been uploaded.
/// This response contains the file ID and owner key, to manage the file.
///
/// It also contains the download URL, although an additional secret is
/// required.
///
/// The download URL can be generated using `download_url()` which will
/// include the required secret in the URL.
#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    /// The file ID.
    id: String,

    /// The URL the file is reachable at.
    /// This includes the file ID, but does not include the secret.
    url: String,

    /// The owner key, used to do further file modifications.
    owner: String,
}

impl UploadResponse {
    /// Convert this response into a file object.
    ///
    /// The `host` and `secret` must be given.
    pub fn into_file(self, host: String, secret: Vec<u8>) -> File {
        File::new_now(
            self.id,
            host,
            self.url,
            secret,
            self.owner,
        )
    }
}
