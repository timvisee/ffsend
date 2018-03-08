use super::super::b64;

/// A struct representing an uploaded file on a Send host.
///
/// The struct contains the file ID, the file URL, the key that is required
/// in combination with the file, and the owner key.
#[derive(Debug)]
pub struct File {
    /// The ID of the file on that server.
    id: String,

    /// The host the file was uploaded to.
    host: String,

    // TODO: a date the file was created at

    /// The file URL that was provided by the server.
    url: String,

    /// The secret key that is required to download the file.
    secret: Vec<u8>,

    /// The owner key, that can be used to manage the file on the server.
    owner_key: String,
}

impl File {
    /// Construct a new file.
    pub fn new(
        id: String,
        host: String,
        url: String,
        secret: Vec<u8>,
        owner_key: String,
    ) -> Self {
        File {
            id,
            host,
            url,
            secret,
            owner_key,
        }
    }

    /// Construct a new file, that was created at this exact time.
    pub fn new_now(
        id: String,
        host: String,
        url: String,
        secret: Vec<u8>,
        owner_key: String,
    ) -> Self {
        Self::new(
            id,
            host,
            url,
            secret,
            owner_key,
        )
    }

    /// Get the download URL of the file, with the secret key included.
    pub fn download_url(&self) -> String {
        format!("{}#{}", self.url, b64::encode(&self.secret))
    }
}
