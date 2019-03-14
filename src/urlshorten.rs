//! URL shortening mechanics.

use ffsend_api::{
    url::{self, Url},
    reqwest::Client,
};
use urlshortener::{
    providers::{Provider, self},
    request::{Method, Request},
};

/// Shorten the given URL.
pub fn shorten(client: &Client, url: &str) -> String {
    // TODO: allow selecting other shorteners
    request(client, providers::request(url, &Provider::IsGd))
}

/// Shorten the given URL.
pub fn shorten_url(client: &Client, url: &Url) -> Result<Url, url::ParseError> {
    Url::parse(&shorten(client, url.as_str()))
}

/// Do the request as given, return the response.
fn request(client: &Client, req: Request) -> String {
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

    builder.send().expect("failed to send shorten request").text().expect("failed to get text")
}
