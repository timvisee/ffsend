pub mod api;
pub mod basic_auth;
pub mod download_limit;
pub mod expiry_time;
pub mod gen_passphrase;
pub mod host;
pub mod owner;
pub mod password;
pub mod url;

// Re-export to arg module
pub use self::api::ArgApi;
pub use self::basic_auth::ArgBasicAuth;
pub use self::download_limit::ArgDownloadLimit;
pub use self::expiry_time::ArgExpiryTime;
pub use self::gen_passphrase::ArgGenPassphrase;
pub use self::host::ArgHost;
pub use self::owner::ArgOwner;
pub use self::password::ArgPassword;
pub use self::url::ArgUrl;

use clap::{Arg, ArgMatches};

/// A generic trait, for a reusable command argument struct.
/// The `CmdArgFlag` and `CmdArgOption` traits further specify what kind of
/// argument this is.
pub trait CmdArg {
    /// Get the argument name that is used as main identifier.
    fn name() -> &'static str;

    /// Build the argument.
    fn build<'a, 'b>() -> Arg<'a, 'b>;
}

/// This `CmdArg` specification defines that this argument may be tested as
/// flag. This will allow to test whether the flag is present in the given
/// matches.
pub trait CmdArgFlag: CmdArg {
    /// Check whether the argument is present in the given matches.
    fn is_present<'a>(matches: &ArgMatches<'a>) -> bool {
        matches.is_present(Self::name())
    }
}

/// This `CmdArg` specification defines that this argument may be tested as
/// option. This will allow to fetch the value of the argument.
pub trait CmdArgOption<'a>: CmdArg {
    /// The type of the argument value.
    type Value;

    /// Get the argument value.
    fn value<'b: 'a>(matches: &'a ArgMatches<'b>) -> Self::Value;

    /// Get the raw argument value, as a string reference.
    fn value_raw<'b: 'a>(matches: &'a ArgMatches<'b>) -> Option<&'a str> {
        matches.value_of(Self::name())
    }
}
