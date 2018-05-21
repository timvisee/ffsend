use ffsend_api::url::{ParseError, Url};

/// Parse the given host string, into an URL.
pub fn parse_host(host: &str) -> Result<Url, HostError> {
    // Trim
    let host = host.trim();

    // Make sure a valid scheme is prefixed
    if !host.starts_with("https://") && !host.starts_with("http://") {
        return Err(HostError::Scheme);
    }

    // Parse the URL, and map the errors
    Url::parse(host).map_err(|err| match err {
        ParseError::EmptyHost => HostError::Empty,
        ParseError::InvalidPort => HostError::Port,
        ParseError::InvalidIpv4Address => HostError::Ipv4,
        ParseError::InvalidIpv6Address => HostError::Ipv6,
        ParseError::InvalidDomainCharacter => HostError::DomainCharacter,
        ParseError::RelativeUrlWithoutBase => HostError::NoBase,
        err => HostError::Other(err),
    })
}

/// An error that has occurred while parsing a host.
#[derive(Debug, Fail)]
pub enum HostError {
    /// The URL scheme is missing or invalid.
    #[fail(display = "the URL must have the 'https://' or 'http://' scheme")]
    Scheme,

    /// The host address is empty.
    #[fail(display = "empty host address")]
    Empty,

    /// The port number is invalid.
    #[fail(display = "invalid port")]
    Port,

    /// The given IPv4 styled address is invalid.
    #[fail(display = "invalid IPv4 address in the host")]
    Ipv4,

    /// The given IPv6 styled address is invalid.
    #[fail(display = "invalid IPv6 address in the host")]
    Ipv6,

    /// The domain contains an invalid character.
    #[fail(display = "invalid character in the domain")]
    DomainCharacter,

    /// The base host is missing from the host URL.
    #[fail(display = "missing host in the host URL")]
    NoBase,

    /// Failed to parse the host URL due to another reason.
    #[fail(display = "could not parse host URL")]
    Other(#[cause] ParseError),
}
