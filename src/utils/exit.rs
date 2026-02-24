//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, Script, ShellCore};
use crate::utils::ExecError;
use std::process;

pub fn normal(core: &mut ShellCore) -> ! {
    run_script(core);

    core.write_history_to_file();
    process::exit(core.db.exit_status % 256)
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
        }
        Err(e) => {
            e.print(core);
        }
        Ok(None) => {}
    };
}

/* error at exec */
pub fn arg_list_too_long(command_name: &str, core: &mut ShellCore) -> ! {
    ExecError::ArgListTooLong(command_name.to_string()).print(core);
    process::exit(126)
}

pub fn permission_denied(command_name: &str, core: &mut ShellCore) -> ! {
    ExecError::PermissionDenied(command_name.to_string()).print(core);
    process::exit(126)
}

pub fn not_found(command_name: &str, core: &mut ShellCore) -> ! {
    ExecError::CommandNotFound(command_name.to_string()).print(core);
    process::exit(127)
}

pub fn internal(s: &str) -> ! {
    panic!("SUSH INTERNAL ERROR: {s}")
}

pub fn check_e_option(core: &mut ShellCore) {
    if core.db.exit_status != 0 && core.db.flags.contains("e") && !core.suspend_e_option {
        normal(core);
    }
}
