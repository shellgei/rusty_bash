//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
use crate::utils::{random, clock};
use std::{env, process};
use super::getter;

pub fn initialize(data: &mut DataBase) {
    data.exit_status = 0;

    data.set_param("$", &process::id().to_string()).unwrap();
    data.set_param("BASHPID", &process::id().to_string()).unwrap();
    data.set_param("BASH_SUBSHELL", "0").unwrap();
    data.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string())).unwrap();
    data.set_param("OPTIND", "1").unwrap();

    data.set_special_variable("SRANDOM", random::get_srandom);
    data.set_special_variable("RANDOM", random::get_random);
    data.set_special_variable("EPOCHSECONDS", clock::get_epochseconds);
    data.set_special_variable("EPOCHREALTIME", clock::get_epochrealtime);
    data.set_special_variable("SECONDS", clock::get_seconds);

    getter::special_variable(data, "SECONDS");

    data.set_array("FUNCNAME", vec![], None).unwrap();
}

pub fn flag(db: &mut DataBase, name: &str, flag: char) {
    let layer = db.position_parameters.len() - 1;
    let rf = &mut db.param_options[layer];
    match rf.get_mut(name) {
        Some(d) => d.push(flag),
        None => {rf.insert(name.to_string(), flag.to_string()); },
    }
}
