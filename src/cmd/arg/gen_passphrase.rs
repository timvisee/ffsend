use clap::Arg;
use chbs;

use super::{CmdArg, CmdArgFlag};

/// The number of words the passphrase must consist of.
const PASSPHRASE_WORDS: usize = 4;

/// The passphrase generation argument.
pub struct ArgGenPassphrase { }

impl ArgGenPassphrase {
    /// Generate a cryptographically secure passphrase that is easily
    /// rememberable using diceware.
    pub fn gen_passphrase() -> String {
        chbs::passphrase(PASSPHRASE_WORDS)
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
            .help("Protect the file with a generated passphrase")
    }
}

impl CmdArgFlag for ArgGenPassphrase { }
