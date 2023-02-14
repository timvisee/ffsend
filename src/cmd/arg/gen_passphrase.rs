use chbs;
use clap::Arg;

use super::{CmdArg, CmdArgFlag};

/// The passphrase generation argument.
pub struct ArgGenPassphrase {}

impl ArgGenPassphrase {
    /// Generate a cryptographically secure passphrase that is easily
    /// remembered using diceware.
    pub fn gen_passphrase() -> String {
        chbs::passphrase()
    }
}

impl CmdArg for ArgGenPassphrase {
    fn name() -> &'static str {
        "gen-passphrase"
    }

    fn build<'b, 'c>() -> Arg<'b, 'c> {
        Arg::with_name("gen-passphrase")
            .long("gen-passphrase")
            .alias("gen-password")
            .alias("generate-passphrase")
            .alias("generate-password")
            .short("P")
            .conflicts_with("password")
            .help("Protect the file with a generated passphrase")
    }
}

impl CmdArgFlag for ArgGenPassphrase {}
