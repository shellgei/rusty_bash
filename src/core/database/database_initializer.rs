//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use super::data::epochrealtime::EpochRealTime;
use super::data::epochseconds::EpochSeconds;
use super::data::random::RandomVar;
use super::data::seconds::Seconds;
use super::data::srandom::SRandomVar;
use std::{env, process};
use crate::core::DataBase;
use nix::unistd;

pub fn initialize(db: &mut DataBase) -> Result<(), String> {
    db.exit_status = 0;

    db.set_param("$", &process::id().to_string(), None)?;
    db.set_param("BASHPID", &process::id().to_string(), None)?;
    db.set_param("BASH_SUBSHELL", "0", None)?;
    db.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()), None)?;
    db.set_param("OPTIND", "1", None)?;
    db.set_param("IFS", " \t\n", None)?;

    db.init_as_num("UID", &unistd::getuid().to_string(), None)?;
    db.set_flag("UID", 'r', 0);

    db.params[0].insert("RANDOM".to_string(), Box::new(RandomVar::new()));
    db.params[0].insert("SRANDOM".to_string(), Box::new(SRandomVar::new()));
    db.params[0].insert("SECONDS".to_string(), Box::new(Seconds::new()));
    db.params[0].insert("EPOCHSECONDS".to_string(), Box::new(EpochSeconds::new()));
    db.params[0].insert("EPOCHREALTIME".to_string(), Box::new(EpochRealTime::new()));

    db.set_array("FUNCNAME", None, None, false)?;
    db.set_array("BASH_SOURCE", Some(vec![]), None, false)?;
    db.set_array("BASH_ARGC", Some(vec![]), None, false)?;
    db.set_array("BASH_ARGV", Some(vec![]), None, false)?;
    db.set_array("BASH_LINENO", Some(vec![]), None, false)?;
    db.set_array("DIRSTACK", Some(vec![]), None, false)?;
    db.set_assoc("BASH_ALIASES", None, true, false)?;
    db.set_assoc("BASH_CMDS", None, true, false)?;
    Ok(())
}
