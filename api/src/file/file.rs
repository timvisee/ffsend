extern crate chrono;
extern crate regex;

use url::{
    ParseError as UrlParseError,
    Url,
};
use self::chrono::{DateTime, Utc};
use self::regex::Regex;

use crypto::b64;

/// A pattern for Send download URL paths, capturing the file ID.
// TODO: match any sub-path?
// TODO: match URL-safe base64 chars for the file ID?
// TODO: constrain the ID length?
const DOWNLOAD_PATH_PATTERN: &'static str = r"$/?download/([[:alnum:]]{8,}={0,3})/?^";

/// A pattern for Send download URL fragments, capturing the file secret.
// TODO: constrain the secret length?
const DOWNLOAD_FRAGMENT_PATTERN: &'static str = r"$([a-zA-Z0-9-_+\/]+)?\s*^";

/// A struct representing an uploaded file on a Send host.
///
/// The struct contains the file ID, the file URL, the key that is required
/// in combination with the file, and the owner key.
#[derive(Debug)]
pub struct File {
    /// The ID of the file on that server.
    id: String,

    /// The time the file was uploaded at.
    time: DateTime<Utc>,

    /// The host the file was uploaded to.
    host: Url,

    /// The file URL that was provided by the server.
    url: Url,

    /// The secret key that is required to download the file.
    secret: Vec<u8>,

    /// The owner key, that can be used to manage the file on the server.
    owner_key: String,
}

impl File {
    /// Construct a new file.
    pub fn new(
        id: String,
        time: DateTime<Utc>,
        host: Url,
        url: Url,
        secret: Vec<u8>,
        owner_key: String,
    ) -> Self {
        Self {
            id,
            time,
            host,
            url,
            secret,
            owner_key,
        }
    }

    /// Construct a new file, that was created at this exact time.
    pub fn new_now(
        id: String,
        host: Url,
        url: Url,
        secret: Vec<u8>,
        owner_key: String,
    ) -> Self {
        Self::new(
            id,
            Utc::now(),
            host,
            url,
            secret,
            owner_key,
        )
    }

    /// Get the raw secret.
    pub fn secret_raw(&self) -> &Vec<u8> {
        // A secret must have been set
        if !self.has_secret() {
            // TODO: don't panic, return an error instead
            panic!("missing secret");
        }

        &self.secret
    }

    /// Get the secret as base64 encoded string.
    pub fn secret(&self) -> String {
        b64::encode(self.secret_raw())
    }

    /// Check whether a file secret is set.
    /// This secret must be set to decrypt a downloaded Send file.
    pub fn has_secret(&self) -> bool {
        !self.secret.is_empty()
    }

    /// Get the download URL of the file.
    /// Set `secret` to `true`, to include it in the URL if known.
    pub fn download_url(&self, secret: bool) -> Url {
        // Get the download URL, and add the secret fragment
        let mut url = self.url.clone();
        if secret && self.has_secret() {
            url.set_fragment(Some(&self.secret()));
        } else {
            url.set_fragment(None);
        }

        url
    }
}

// TODO: merge this struct with `File`.
pub struct DownloadFile {
    /// The ID of the file on that server.
    id: String,

    /// The host the file was uploaded to.
    host: Url,

    /// The file URL that was provided by the server.
    url: Url,

    /// The secret key that is required to download the file.
    secret: Vec<u8>,
}

impl DownloadFile {
    /// Construct a new instance.
    pub fn new(
        id: String,
        host: Url,
        url: Url,
        secret: Vec<u8>,
    ) -> Self {
        Self {
            id,
            host,
            url,
            secret,
        }
    }

    /// Try to parse the given Send download URL.
    ///
    /// The given URL is matched against a Send download URL pattern,
    /// this does not check whether the host is a valid and online Send host.
    ///
    /// If the URL fragmet contains a file secret, it is also parsed.
    /// If it does not, the secret is left empty and must be specified
    /// manually.
    pub fn parse_url(url: String) -> Result<DownloadFile, FileParseError> {
        // Try to parse as an URL
        let url = Url::parse(&url)
            .map_err(|err| FileParseError::UrlFormatError(err))?;

        // Build the host
        let mut host = url.clone();
        host.set_fragment(None);
        host.set_query(None);
        host.set_path("");

        // TODO: remove this after debugging
        println!("DEBUG: Extracted host: {}", host);

        // Validate the path, get the file ID
        let re_path = Regex::new(DOWNLOAD_PATH_PATTERN).unwrap();
        let id = re_path.captures(url.path())
            .ok_or(FileParseError::InvalidDownloadUrl)?[1]
            .trim()
            .to_owned();

        // Get the file secret
        let mut secret = Vec::new();
        if let Some(fragment) = url.fragment() {
            let re_fragment = Regex::new(DOWNLOAD_FRAGMENT_PATTERN).unwrap();
            if let Some(raw) = re_fragment.captures(fragment)
                .ok_or(FileParseError::InvalidSecret)?
                .get(1)
            {
                secret = b64::decode(raw.as_str().trim())
                        .map_err(|_| FileParseError::InvalidSecret)?
            }
        }

        // Construct the file
        Ok(Self::new(
            id,
            host,
            url,
            secret,
        ))
    }

    /// Get the raw secret.
    pub fn secret_raw(&self) -> &Vec<u8> {
        // A secret must have been set
        if !self.has_secret() {
            // TODO: don't panic, return an error instead
            panic!("missing secret");
        }

        &self.secret
    }

    /// Get the secret as base64 encoded string.
    pub fn secret(&self) -> String {
        b64::encode(self.secret_raw())
    }

    /// Check whether a file secret is set.
    /// This secret must be set to decrypt a downloaded Send file.
    pub fn has_secret(&self) -> bool {
        !self.secret.is_empty()
    }

    /// Set the secret for this file.
    /// An empty vector will clear the secret.
    pub fn set_secret(&mut self, secret: Vec<u8>) {
        self.secret = secret;
    }

    /// Get the download URL of the file.
    /// Set `secret` to `true`, to include it in the URL if known.
    pub fn download_url(&self, secret: bool) -> Url {
        // Get the download URL, and add the secret fragment
        let mut url = self.url.clone();
        if secret && self.has_secret() {
            url.set_fragment(Some(&self.secret()));
        } else {
            url.set_fragment(None);
        }

        url
    }
}

pub enum FileParseError {
    /// An URL format error.
    UrlFormatError(UrlParseError),

    /// An error for an invalid download URL format.
    InvalidDownloadUrl,

    /// An error for an invalid secret format, if an URL fragmet exists.
    InvalidSecret,
}
