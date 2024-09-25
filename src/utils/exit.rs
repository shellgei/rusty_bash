//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::error;
use std::process;

pub fn normal(core: &mut ShellCore) -> ! {
    core.write_history_to_file();

    let es_str = core.data.get_param("?");
    let exit_status = match es_str.parse::<i32>() {
        Ok(n)  => n%256,
        Err(_) => {
            let msg = format!("exit: {}: numeric argument required", es_str);
            error::print(&msg, core, true);
            2
        },
    };

    process::exit(exit_status)
}

/* error at exec */
fn command_error_exit(name: &str, core: &mut ShellCore, msg: &str, exit_status: i32) -> ! {
    let msg = format!("{}: {}", name, msg);
    error::print(&msg, core, true);
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

