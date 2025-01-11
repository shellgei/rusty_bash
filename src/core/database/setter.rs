//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::core::database::SpecialData;
use crate::utils::{random, clock};
use std::{env, process};
use super::getter;

pub fn initialize(db: &mut DataBase) -> Result<(), String> {
    db.exit_status = 0;

    db.set_param("$", &process::id().to_string(), None)?;
    db.set_param("BASHPID", &process::id().to_string(), None)?;
    db.set_param("BASH_SUBSHELL", "0", None)?;
    db.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()), None)?;
    db.set_param("OPTIND", "1", None)?;

    SpecialData::set(&mut db.params[0], "SRANDOM", random::get_srandom)?;
    SpecialData::set(&mut db.params[0], "RANDOM", random::get_random)?;
    SpecialData::set(&mut db.params[0], "EPOCHSECONDS", clock::get_epochseconds)?;
    SpecialData::set(&mut db.params[0], "EPOCHREALTIME", clock::get_epochrealtime)?;
    SpecialData::set(&mut db.params[0], "SECONDS", clock::get_seconds)?;

    getter::special_variable(db, "SECONDS");

    db.set_array("FUNCNAME", vec![], None)?;
    Ok(())
}

pub fn flag(db: &mut DataBase, name: &str, flag: char) {
    let layer = db.position_parameters.len() - 1;
    let rf = &mut db.param_options[layer];
    match rf.get_mut(name) {
        Some(d) => d.push(flag),
        None => {rf.insert(name.to_string(), flag.to_string()); },
    }
}
