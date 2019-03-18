pub mod debug;
pub mod delete;
pub mod download;
pub mod exists;
pub mod generate;
#[cfg(feature = "history")]
pub mod history;
pub mod info;
pub mod main;
pub mod params;
pub mod password;
pub mod upload;
pub mod version;

// Re-export to matcher module
pub use self::debug::DebugMatcher;
pub use self::delete::DeleteMatcher;
pub use self::download::DownloadMatcher;
pub use self::exists::ExistsMatcher;
pub use self::generate::GenerateMatcher;
#[cfg(feature = "history")]
pub use self::history::HistoryMatcher;
pub use self::info::InfoMatcher;
pub use self::main::MainMatcher;
pub use self::params::ParamsMatcher;
pub use self::password::PasswordMatcher;
pub use self::upload::{CopyMode, UploadMatcher};
pub use self::version::VersionMatcher;

use clap::ArgMatches;

pub trait Matcher<'a>: Sized {
    // Construct a new matcher instance from these argument matches.
    fn with(matches: &'a ArgMatches) -> Option<Self>;
}
