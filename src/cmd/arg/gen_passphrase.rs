use chbs::{config::BasicConfig, prelude::*, word::WordList};
use clap::Arg;

use super::{CmdArg, CmdArgFlag};

/// How many words the passphrase should consist of.
const PASSPHRASE_WORDS: usize = 5;

/// The passphrase generation argument.
pub struct ArgGenPassphrase {}

impl ArgGenPassphrase {
    /// Generate a cryptographically secure passphrase that is easily
    /// rememberable using diceware.
    pub fn gen_passphrase() -> String {
        let mut config = BasicConfig::default();
        config.words = PASSPHRASE_WORDS;
        config.word_provider = WordList::builtin_eff_general_short().sampler();
        config.to_scheme().generate()
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

impl CmdArgFlag for ArgGenPassphrase {}
