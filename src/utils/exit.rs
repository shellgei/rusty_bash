//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error;
use std::process;

pub fn normal(core: &mut ShellCore) -> ! {
    core.write_history_to_file();
    process::exit(core.db.exit_status%256)
}

/* error at exec */
fn command_error_exit(name: &str, core: &mut ShellCore, msg: &str, exit_status: i32) -> ! {
    let msg = format!("{}: {}", name, msg);
    error::print(&msg, core);
    process::exit(exit_status)
}

pub fn arg_list_too_long(command_name: &str, core: &mut ShellCore) -> ! {
    command_error_exit(command_name, core, "Arg list too long", 126)
}

pub fn permission_denied(command_name: &str, core: &mut ShellCore) -> ! {
    command_error_exit(command_name, core, "Permission denied", 126)
}

pub fn not_found(command_name: &str, core: &mut ShellCore) -> ! {
    command_error_exit(command_name, core, "command not found", 127)
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

