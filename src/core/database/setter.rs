//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::core::database::SpecialData;
use crate::utils::{random, clock};
use std::{env, process};
use super::getter;

pub fn initialize(data: &mut DataBase) -> Result<(), String> {
    data.exit_status = 0;

    data.set_param("$", &process::id().to_string())?;
    data.set_param("BASHPID", &process::id().to_string())?;
    data.set_param("BASH_SUBSHELL", "0")?;
    data.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()))?;
    data.set_param("OPTIND", "1")?;

    SpecialData::set(&mut data.params[0], "SRANDOM", random::get_srandom)?;
    SpecialData::set(&mut data.params[0], "RANDOM", random::get_random)?;
    SpecialData::set(&mut data.params[0], "EPOCHSECONDS", clock::get_epochseconds)?;
    SpecialData::set(&mut data.params[0], "EPOCHREALTIME", clock::get_epochrealtime)?;
    SpecialData::set(&mut data.params[0], "SECONDS", clock::get_seconds)?;

    getter::special_variable(data, "SECONDS");

    data.set_array("FUNCNAME", vec![], None)?;
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

/*
pub fn set_special_variable(db: &mut DataBase, key: &str, f: fn(&mut Vec<String>)-> String) {
    db.params[0].insert( key.to_string(), Box::new(SpecialData::from(f)) );
}*/
