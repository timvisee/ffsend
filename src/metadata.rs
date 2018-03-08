extern crate serde_json;

use std::fmt;

use hyper::error::Error as HyperError;
use mime_guess::Mime;
use reqwest::header::{
    Formatter as HeaderFormatter,
    Header,
    Raw,
};

use b64;

/// File metadata, which is send to the server.
#[derive(Serialize)]
pub struct Metadata {
    /// The input vector.
    iv: String,

    /// The file name.
    name: String,

    /// The file mimetype.
    #[serde(rename="type")]
    mime: String,
}

impl Metadata {
    /// Construct metadata from the given properties.
    ///
    /// Parameters:
    /// * iv: initialisation vector
    /// * name: file name
    /// * mime: file mimetype
    pub fn from(iv: &[u8], name: String, mime: Mime) -> Self {
        Metadata {
            iv: b64::encode(iv),
            name,
            mime: mime.to_string(),
        }
    }

    /// Convert this structure to a JSON string.
    pub fn to_json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

/// A X-File-Metadata header for reqwest, that is used to pass encrypted
/// metadata to the server.
///
/// The encrypted metadata (bytes) is base64 encoded when constructing this
/// header using `from`.
#[derive(Clone)]
pub struct XFileMetadata {
    /// The metadata, as a base64 encoded string.
    metadata: String,
}

impl XFileMetadata {
    /// Construct the header from the given encrypted metadata.
    pub fn from(bytes: &[u8]) -> Self {
        XFileMetadata {
            metadata: b64::encode(bytes),
        }
    }
}

/// Make this struct usable as reqwest header.
impl Header for XFileMetadata {
    fn header_name() -> &'static str {
        "X-File-Metadata"
    }

    fn parse_header(_raw: &Raw) -> Result<Self, HyperError> {
        // TODO: implement this some time
        unimplemented!();
    }

    fn fmt_header(&self, f: &mut HeaderFormatter) -> fmt::Result {
        // TODO: is this encoding base64 for us?
        f.fmt_line(&self.metadata)
    }
}
