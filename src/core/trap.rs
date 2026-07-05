//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, Script, ShellCore};
use super::{Arc, AtomicBool, HashMap};

pub const SPECIAL_TRAP_MIN: i32 = 1000;
pub const ERROR: i32 = 1000;
pub const RETURN: i32 = 1001;
pub const DEBUG: i32 = 1002;
pub const SPECIAL_TRAP_MAX: i32 = DEBUG;

#[derive(Default)]
pub struct Trap {
    pub list: HashMap<i32, String>,
    pub trapped: Vec<(Arc<AtomicBool>, String)>,
    pub exit_script_run: bool,
    pub debug_script_run: bool,
    pub error_script_run: bool,
    pub return_script_run: bool,
}

impl Trap {
    pub fn clear_for_subshell(&mut self, extdebug: bool) {
        for n in [0, DEBUG, RETURN, ERROR] {
            if self.list.contains_key(&n) {
                self.list.remove(&n);
            }

            if extdebug {
                return;
            }
        }
    }
}

fn run_(feeder: &mut Feeder, core: &mut ShellCore) {
    match Script::parse(feeder, core, true) {
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
        }
        Err(e) => e.print(core),
        Ok(None) => {}
    }
}

fn run(n: i32, core: &mut ShellCore) {
    let mut feeder = Feeder::new(&core.trap.list[&n]);
    run_(&mut feeder, core);
}

pub fn exit(core: &mut ShellCore) {
    if ! core.trap.list.contains_key(&0) 
    || core.trap.exit_script_run {
        return;
    }

    let exit_status_bkup = core.db.exit_status;
    core.trap.exit_script_run = true;
    run(0, core);
    core.db.exit_status = exit_status_bkup;
}

pub fn error(core: &mut ShellCore) {
    if ! core.trap.list.contains_key(&ERROR) {
        return;
    }

    core.trap.error_script_run = true;
    run(ERROR, core);
    core.db.exit_status = 0;
    core.trap.error_script_run = false;
}

pub fn debug(core: &mut ShellCore) -> bool {
    if ! core.trap.list.contains_key(&DEBUG) 
    || core.trap.debug_script_run {
        return true;
    }

    core.trap.debug_script_run = true;
    let bkup = core.trap.list[&DEBUG].clone();
    core.trap.list.remove(&DEBUG);
    let mut feeder = Feeder::new(&bkup);
    let lineno = core.db.get_param("LINENO").unwrap_or("0".to_string()).parse::<usize>().unwrap_or(0);
    feeder.lineno += lineno - 1;
    run_(&mut feeder, core);

    core.trap.list.insert(DEBUG, bkup);
    core.trap.debug_script_run = false;

    if core.db.exit_status != 0 {
        if core.shopts.query("extdebug") {
            return false;
        }
    }

    true
}

pub fn r#return(core: &mut ShellCore) {
    if ! core.trap.list.contains_key(&RETURN) 
    || core.trap.return_script_run {
        return;
    }

    core.trap.return_script_run = true;
    let bkup = core.trap.list[&RETURN].clone();
    core.trap.list.remove(&RETURN);
    let mut feeder = Feeder::new(&bkup);
    //let lineno = core.db.get_param("LINENO").unwrap_or("0".to_string()).parse::<usize>().unwrap_or(0);
    //feeder.lineno += lineno - 1;
    run_(&mut feeder, core);
    core.trap.list.insert(RETURN, bkup);
    core.trap.return_script_run = false;
}

