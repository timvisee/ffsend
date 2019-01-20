use clap::{App, SubCommand};

use crate::cmd::arg::{ArgGenPassphrase, ArgOwner, ArgPassword, ArgUrl, CmdArg};

/// The password command definition.
pub struct CmdPassword;

impl CmdPassword {
    pub fn build<'a, 'b>() -> App<'a, 'b> {
        SubCommand::with_name("password")
            .about("Change the password of a shared file")
            .visible_alias("pass")
            .visible_alias("p")
            .arg(ArgUrl::build())
            .arg(ArgPassword::build().help("Specify a password, do not prompt"))
            .arg(ArgGenPassphrase::build())
            .arg(ArgOwner::build())
    }
}
