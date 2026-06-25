//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, Script, ShellCore};
use super::{Arc, AtomicBool, HashMap};

pub const SPECIAL_TRAP_MIN: i32 = 1000;
pub const ERROR: i32 = 1000;
pub const DEBUG: i32 = 1001;
pub const SPECIAL_TRAP_MAX: i32 = DEBUG;

#[derive(Default)]
pub struct Trap {
    //pub list: Vec<(i32, String)>,
    pub list: HashMap<i32, String>,
    pub trapped: Vec<(Arc<AtomicBool>, String)>,
    pub exit_script: String,
    pub exit_script_run: bool,
    pub debug_script: String,
    pub debug_script_run: bool,
    pub error_script: String,
    pub error_script_run: bool,
}

impl Trap {
    pub fn clear_for_subshell(&mut self) {
        self.exit_script.clear();
        self.debug_script.clear();
    }
}

pub fn exit(core: &mut ShellCore) {
    if core.trap.exit_script_run {
        return;
    }

    let exit_status_bkup = core.db.exit_status;
    core.trap.exit_script_run = true;
    if core.trap.exit_script.is_empty() {
        return;
    }

    let mut feeder = Feeder::new(&core.trap.exit_script);
    //let mut feeder = Feeder::new(&core.trap.list[0]);
    match Script::parse(&mut feeder, core, true) {
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
        }
        Err(e) => {
            e.print(core);
        }
        Ok(None) => {}
    };

    core.db.exit_status = exit_status_bkup;
}

pub fn error(core: &mut ShellCore) {
    if core.trap.error_script.is_empty() {
        return;
    }

    core.trap.error_script_run = true;
    let mut feeder = Feeder::new(&core.trap.error_script);
    match Script::parse(&mut feeder, core, true) {
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
        }
        Err(e) => {
            e.print(core);
        }
        Ok(None) => {}
    };

    core.db.exit_status = 0;
    core.trap.error_script_run = false;
}

pub fn debug(core: &mut ShellCore) {
    if core.trap.debug_script.is_empty() 
    || core.trap.debug_script_run {
        return;
    }

    core.trap.debug_script_run = true;
    let mut feeder = Feeder::new(&core.trap.debug_script);
    let lineno = core.db.get_param("LINENO").unwrap_or("0".to_string()).parse::<usize>().unwrap_or(0);
    feeder.lineno += lineno - 1;
    let bkup = core.trap.debug_script.clone();
    core.trap.debug_script.clear();
    match Script::parse(&mut feeder, core, true) {
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
        }
        Err(e) => {
            e.print(core);
        }
        Ok(None) => {}
    };

    //core.db.exit_status = 0;
    core.trap.debug_script = bkup;
    core.trap.debug_script_run = false;
}

/*
pub fn r#return(core: &mut ShellCore) {
    if core.error_script.is_empty() {
        return;
    }

    core.error_script_run = true;
    let mut feeder = Feeder::new(&core.error_script);
    match Script::parse(&mut feeder, core, true) {
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
        }
        Err(e) => {
            e.print(core);
        }
        Ok(None) => {}
    };

    core.db.exit_status = 0;
    core.error_script_run = false;
}
*/
