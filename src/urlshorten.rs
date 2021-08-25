//! URL shortening mechanics.

use ffsend_api::{
    api::request::{ensure_success, ResponseError},
    client::Client,
    reqwest,
    url::{self, Url},
};
use urlshortener::{
    providers::{self, Provider},
    request::{Method, Request},
};

/// An URL shortening result.
type Result<T> = ::std::result::Result<T, Error>;

/// Shorten the given URL.
pub fn shorten(client: &Client, url: &str) -> Result<String> {
    // TODO: allow selecting other shorteners
    request(client, providers::request(url, &Provider::IsGd))
}

/// Shorten the given URL.
pub fn shorten_url(client: &Client, url: &Url) -> Result<Url> {
    Url::parse(&shorten(client, url.as_str())?).map_err(|err| err.into())
}

/// Do the request as given, return the response.
fn request(client: &Client, req: Request) -> Result<String> {
    // Start the request builder
    let mut builder = match req.method {
        Method::Get => client.get(&req.url),
        Method::Post => client.post(&req.url),
    };

    // Define the custom user agent
    if let Some(_agent) = req.user_agent.clone() {
        // TODO: implement this
        // builder.header(header::UserAgent::new(agent.0));
        panic!("Custom UserAgent for URL shortener not yet implemented");
    }

    // Define the custom content type
    if let Some(_content_type) = req.content_type {
        // TODO: implement this
        // match content_type {
        //     ContentType::Json => builder.header(header::ContentType::json()),
        //     ContentType::FormUrlEncoded => {
        //         builder.header(header::ContentType::form_url_encoded())
        //     }
        // };
        panic!("Custom UserAgent for URL shortener not yet implemented");
    }

    // Define the custom body
    if let Some(body) = req.body.clone() {
        builder = builder.body(body);
    }

    // Send the request, ensure success
    let response = builder.send().map_err(Error::Request)?;
    ensure_success(&response)?;

    // Respond with the body text
    response.text().map_err(Error::Malformed)
}

/// An URL shortening error.
#[derive(Debug, Fail)]
pub enum Error {
    /// Failed to send the shortening request.
    #[fail(display = "failed to send URL shorten request")]
    Request(#[cause] reqwest::Error),

    /// The server responded with a bad response.
    #[fail(display = "failed to shorten URL, got bad response")]
    Response(#[cause] ResponseError),

    /// The server responded with a malformed response.
    #[fail(display = "failed to shorten URL, got malformed response")]
    Malformed(#[cause] reqwest::Error),

    /// An error occurred while parsing the shortened URL.
    #[fail(display = "failed to shorten URL, could not parse URL")]
    Url(#[cause] url::ParseError),
}

impl From<url::ParseError> for Error {
    fn from(err: url::ParseError) -> Self {
        Error::Url(err)
    }
}

impl From<ResponseError> for Error {
    fn from(err: ResponseError) -> Self {
        Error::Response(err)
    }
}
