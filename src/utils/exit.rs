//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error, Feeder, Script, ShellCore};
use std::process;
use std::ffi::CString;

pub fn normal(core: &mut ShellCore) -> ! {
    run_script(core);

    core.write_history_to_file();
    process::exit(core.db.exit_status%256)
}

fn run_script(core: &mut ShellCore) {
    if core.exit_script_run {
        return;
    }

    core.exit_script_run = true;
    if core.exit_script.is_empty() {
        return;
    }

    let mut feeder = Feeder::new(&core.exit_script);
    match Script::parse(&mut feeder, core, true) {
        Ok(Some(mut s)) => {
            if let Err(e) = s.exec(core) {
                e.print(core);
            }
        },
        Err(e) => {e.print(core);},
        Ok(None) => {},
    };
}

/* error at exec */
fn command_error_exit(name: &CString, core: &mut ShellCore, msg: &str, exit_status: i32) -> ! {
    let msg = format!("{}: {}", name.to_str().unwrap(), msg);
    error::print(&msg, core);
    process::exit(exit_status)
}

pub fn arg_list_too_long(command_name: &CString, core: &mut ShellCore) -> ! {
    command_error_exit(command_name, core, "Arg list too long", 126)
}

pub fn permission_denied(command_name: &CString, core: &mut ShellCore) -> ! {
    command_error_exit(command_name, core, "Permission denied", 126)
}

pub fn not_found(command_name: &CString, core: &mut ShellCore) -> ! {
    let msg = "command not found";
    command_error_exit(command_name, core, &msg, 127)
}

pub fn internal(s: &str) -> ! {
    panic!("SUSH INTERNAL ERROR: {}", s)
}

pub fn check_e_option(core: &mut ShellCore) {
    if core.db.exit_status != 0 
    && core.db.flags.contains("e") 
    && ! core.suspend_e_option {
        normal(core);
    }
}
