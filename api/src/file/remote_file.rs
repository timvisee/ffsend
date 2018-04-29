extern crate chrono;
extern crate regex;

use api::url::UrlBuilder;
use url::{
    ParseError as UrlParseError,
    Url,
};
use self::chrono::{DateTime, Duration, Utc};
use self::regex::Regex;
use url_serde;

use config::SEND_DEFAULT_EXPIRE_TIME;
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
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RemoteFile {
    /// The ID of the file on that server.
    id: String,

    /// The time the file was uploaded at, if known.
    upload_at: Option<DateTime<Utc>>,

    /// The time the file will expire at, if known.
    expire_at: Option<DateTime<Utc>>,

    /// The host the file was uploaded to.
    #[serde(with = "url_serde")]
    host: Url,

    /// The file URL that was provided by the server.
    #[serde(with = "url_serde")]
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
        upload_at: Option<DateTime<Utc>>,
        expire_at: Option<DateTime<Utc>>,
        host: Url,
        url: Url,
        secret: Vec<u8>,
        owner_token: Option<String>,
    ) -> Self {
        Self {
            id,
            upload_at,
            expire_at,
            host,
            url,
            secret,
            owner_token,
        }
    }

    /// Construct a new file, that was created at this exact time.
    /// This will set the file expiration time 
    pub fn new_now(
        id: String,
        host: Url,
        url: Url,
        secret: Vec<u8>,
        owner_token: Option<String>,
    ) -> Self {
        // Get the current time
        let now = Utc::now();
        let expire_at = now + Duration::seconds(SEND_DEFAULT_EXPIRE_TIME);

        // Construct and return
        Self::new(
            id,
            Some(now),
            Some(expire_at),
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
            None,
            host,
            url,
            secret,
            owner_token,
        ))
    }

    /// Get the file ID.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the duration the file will expire after,
    /// if an expiry time is known.
    /// Otherwise `None` is returned.
    pub fn expire_duration(&self) -> Option<Duration> {
        self.expire_at.as_ref().map(|time| {
            // Get the current time
            let now = Utc::now();

            // Return the duration if not expired, otherwise return zero
            if time > &now {
                *time - now
            } else {
                Duration::zero()
            }
        })
    }

    /// Set the time this file will expire at.
    /// None may be given if the expire time is unknown.
    pub fn set_expire_at(&mut self, expire_at: Option<DateTime<Utc>>) {
        self.expire_at = expire_at;
    }

    /// Set the time this file will expire at,
    /// based on the given duration from now.
    pub fn set_expire_duration(&mut self, duration: Duration) {
        self.set_expire_at(Some(Utc::now() + duration));
    }

    /// Check whether this file has expired, based on it's expiry property.
    ///
    /// If no expiry time is set (known) for this file,
    /// the `def` value is returned instead.
    pub fn has_expired(&self, def: bool) -> bool {
        match self.expire_at {
            Some(time) => time < Utc::now(),
            None => def,
        }
    }

    /// Get the file URL, provided by the server.
    pub fn url(&self) -> &Url {
        &self.url
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

    /// Get the owner token if set.
    pub fn owner_token_mut(&mut self) -> &mut Option<String> {
        &mut self.owner_token
    }

    /// Set the owner token.
    pub fn set_owner_token(&mut self, token: Option<String>) {
        self.owner_token = token;
    }

    /// Get the host URL for this remote file.
    pub fn host(&self) -> Url {
        self.host.clone()
    }

    /// Build the download URL of the given file.
    /// This URL is identical to the share URL, a term used in this API.
    /// Set `secret` to `true`, to include it in the URL if known.
    pub fn download_url(&self, secret: bool) -> Url {
        UrlBuilder::download(&self, secret)
    }

    /// Merge properties non-existant into this file, from the given other file.
    /// This is ofcourse only done for properties that may be empty.
    ///
    /// The file IDs are not asserted for equality.
    pub fn merge(&mut self, other: &RemoteFile, overwrite: bool) -> bool {
        // Remember whether anything was changed
        let mut changed = false;

        // Set the upload time
        if other.upload_at.is_some() && (self.upload_at.is_none() || overwrite) {
            self.upload_at = other.upload_at.clone();
            changed = true;
        }

        // Set the expire time
        if other.expire_at.is_some() && (self.expire_at.is_none() || overwrite) {
            self.expire_at = other.expire_at.clone();
            changed = true;
        }

        // Set the secret
        if other.has_secret() && (!self.has_secret() || overwrite) {
            self.secret = other.secret_raw().clone();
            changed = true;
        }

        // Set the owner token
        if other.owner_token.is_some() && (self.owner_token.is_none() || overwrite) {
            self.owner_token = other.owner_token.clone();
            changed = true;
        }

        return changed;
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
