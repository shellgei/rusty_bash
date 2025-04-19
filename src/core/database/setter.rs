//SPDXFileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDXLicense-Identifier: BSD-3-Clause

use crate::core::DataBase;
//use crate::utils::clock;
//use super::SpecialData;
use super::data::random::RandomVar;
use super::data::srandom::SRandomVar;
use super::data::seconds::Seconds;
use super::data::epochseconds::EpochSeconds;
use super::data::epochrealtime::EpochRealTime;
use std::{env, process};

pub fn initialize(db: &mut DataBase) -> Result<(), String> {
    db.exit_status = 0;

    db.set_param("$", &process::id().to_string(), None)?;
    db.set_param("BASHPID", &process::id().to_string(), None)?;
    db.set_param("BASH_SUBSHELL", "0", None)?;
    //db.set_param("COMP_WORDBREAKS", "\"'><=;|&(:", None)?;
    db.set_param("HOME", &env::var("HOME").unwrap_or("/".to_string()), None)?;
    db.set_param("OPTIND", "1", None)?;
    db.set_param("IFS", " \t\n", None)?;

    db.params[0].insert( "RANDOM".to_string(), Box::new(RandomVar::new()) );
    db.params[0].insert( "SRANDOM".to_string(), Box::new(SRandomVar::new()) );
    db.params[0].insert( "SECONDS".to_string(), Box::new(Seconds::new()) );
    db.params[0].insert( "EPOCHSECONDS".to_string(), Box::new(EpochSeconds{} ) );
    db.params[0].insert( "EPOCHREALTIME".to_string(), Box::new(EpochRealTime{} ) );

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
