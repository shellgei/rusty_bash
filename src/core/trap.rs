//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, Script, ShellCore};
use super::{Arc, AtomicBool};

#[derive(Default)]
pub struct Trap {
    pub list: Vec<(i32, String)>,
    pub trapped: Vec<(Arc<AtomicBool>, String)>,
    pub exit_script: String,
    pub exit_script_run: bool,
    pub debug_script: String,
    pub debug_script_run: bool,
    pub error_script: String,
    pub error_script_run: bool,
}

    pub fn exit(core: &mut ShellCore) {
    if core.trap_info.exit_script_run {
        return;
    }

    let exit_status_bkup = core.db.exit_status;
    core.trap_info.exit_script_run = true;
    if core.trap_info.exit_script.is_empty() {
        return;
    }

    let mut feeder = Feeder::new(&core.trap_info.exit_script);
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
    if core.trap_info.error_script.is_empty() {
        return;
    }

    core.trap_info.error_script_run = true;
    let mut feeder = Feeder::new(&core.trap_info.error_script);
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
    core.trap_info.error_script_run = false;
}

pub fn debug(core: &mut ShellCore) {
    if core.trap_info.debug_script.is_empty() 
    || core.trap_info.debug_script_run
    || core.is_subshell {
        return;
    }

    core.trap_info.debug_script_run = true;
    let mut feeder = Feeder::new(&core.trap_info.debug_script);
    let lineno = core.db.get_param("LINENO").unwrap_or("0".to_string()).parse::<usize>().unwrap_or(0);
    feeder.lineno += lineno - 1;
    let bkup = core.trap_info.debug_script.clone();
    core.trap_info.debug_script.clear();
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
    core.trap_info.debug_script = bkup;
    core.trap_info.debug_script_run = false;
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
