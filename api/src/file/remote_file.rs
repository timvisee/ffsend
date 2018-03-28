extern crate chrono;
extern crate regex;

use url::{
    ParseError as UrlParseError,
    Url,
};
use self::chrono::{DateTime, Utc};
use self::regex::Regex;

use crypto::b64;

/// A pattern for share URL paths, capturing the file ID.
// TODO: match any sub-path?
// TODO: match URL-safe base64 chars for the file ID?
// TODO: constrain the ID length?
const SHARE_PATH_PATTERN: &'static str = r"^/?download/([[:alnum:]]{8,}={0,3})/?$";

/// A pattern for share URL fragments, capturing the file secret.
// TODO: constrain the secret length?
const SHARE_FRAGMENT_PATTERN: &'static str = r"^([a-zA-Z0-9-_+/]+)?\s*$";

/// A struct representing an uploaded file on a Send host.
///
/// The struct contains the file ID, the file URL, the key that is required
/// in combination with the file, and the owner key.
#[derive(Debug)]
pub struct RemoteFile {
    /// The ID of the file on that server.
    id: String,

    /// The time the file was uploaded at, if known.
    time: Option<DateTime<Utc>>,

    /// The host the file was uploaded to.
    host: Url,

    /// The file URL that was provided by the server.
    url: Url,

    /// The secret key that is required to download the file.
    secret: Vec<u8>,

    /// The owner key, that can be used to manage the file on the server.
    owner_token: Option<String>,
}

impl RemoteFile {
    /// Construct a new file.
    pub fn new(
        id: String,
        time: Option<DateTime<Utc>>,
        host: Url,
        url: Url,
        secret: Vec<u8>,
        owner_token: Option<String>,
    ) -> Self {
        Self {
            id,
            time,
            host,
            url,
            secret,
            owner_token,
        }
    }

    /// Construct a new file, that was created at this exact time.
    pub fn new_now(
        id: String,
        host: Url,
        url: Url,
        secret: Vec<u8>,
        owner_token: Option<String>,
    ) -> Self {
        Self::new(
            id,
            Some(Utc::now()),
            host,
            url,
            secret,
            owner_token,
        )
    }

    /// Try to parse the given share URL.
    ///
    /// The given URL is matched against a share URL pattern,
    /// this does not check whether the host is a valid and online host.
    ///
    /// If the URL fragmet contains a file secret, it is also parsed.
    /// If it does not, the secret is left empty and must be specified
    /// manually.
    ///
    /// An optional owner token may be given.
    pub fn parse_url(url: Url, owner_token: Option<String>)
        -> Result<RemoteFile, FileParseError>
    {
        // Build the host
        let mut host = url.clone();
        host.set_fragment(None);
        host.set_query(None);
        host.set_path("");

        // Validate the path, get the file ID
        let re_path = Regex::new(SHARE_PATH_PATTERN).unwrap();
        let id = re_path.captures(url.path())
            .ok_or(FileParseError::InvalidUrl)?[1]
            .trim()
            .to_owned();

        // Get the file secret
        let mut secret = Vec::new();
        if let Some(fragment) = url.fragment() {
            let re_fragment = Regex::new(SHARE_FRAGMENT_PATTERN).unwrap();
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
            None,
            host,
            url,
            secret,
            owner_token,
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

    /// Get the owner token if set.
    pub fn owner_token(&self) -> Option<&String> {
        self.owner_token.as_ref()
    }

    /// Set the owner token.
    pub fn set_owner_token(&mut self, token: Option<String>) {
        self.owner_token = token;
    }

    /// Get the download URL of the file
    /// This URL is identical to the share URL, a term used in this API.
    /// Set `secret` to `true`, to include it in the URL if known.
    pub fn download_url(&self, secret: bool) -> Url {
        // Get the share URL, and add the secret fragment
        let mut url = self.url.clone();
        if secret && self.has_secret() {
            url.set_fragment(Some(&self.secret()));
        } else {
            url.set_fragment(None);
        }

        url
    }

    /// Get the API metadata URL of the file.
    pub fn api_meta_url(&self) -> Url {
        // Get the share URL, and add the secret fragment
        let mut url = self.url.clone();
        url.set_path(format!("/api/metadata/{}", self.id).as_str());
        url.set_fragment(None);

        url
    }

    /// Get the API download URL of the file.
    pub fn api_download_url(&self) -> Url {
        // Get the share URL, and add the secret fragment
        let mut url = self.url.clone();
        url.set_path(format!("/api/download/{}", self.id).as_str());
        url.set_fragment(None);

        url
    }

    /// Get the API password URL of the file.
    pub fn api_password_url(&self) -> Url {
        // Get the share URL, and add the secret fragment
        let mut url = self.url.clone();
        url.set_path(format!("/api/password/{}", self.id).as_str());
        url.set_fragment(None);

        url
    }
}

#[derive(Debug, Fail)]
pub enum FileParseError {
    /// An URL format error.
    #[fail(display = "Failed to parse remote file, invalid URL format")]
    UrlFormatError(#[cause] UrlParseError),

    /// An error for an invalid share URL format.
    #[fail(display = "Failed to parse remote file, invalid URL")]
    InvalidUrl,

    /// An error for an invalid secret format, if an URL fragmet exists.
    #[fail(display = "Failed to parse remote file, invalid secret in URL")]
    InvalidSecret,
}
