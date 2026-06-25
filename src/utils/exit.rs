//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::utils::{ExecError, InputError};
use crate::ShellCore;
use crate::core::trap;
use nix::sys::signal;
use std::process;

pub fn normal(core: &mut ShellCore) -> ! {
    trap::exit(core);

    if !core.is_subshell {
        core.write_history_to_file();

        for e in core.job_table.iter_mut() {
            if e.coproc_name.is_some() {
                let _ = signal::killpg(e.pids[0], signal::SIGTERM);
            }
        }
    }

    process::exit(core.db.exit_status % 256)
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

pub fn is_a_dir(command_name: &str, core: &mut ShellCore) -> ! {
    ExecError::IsDir(command_name.to_string()).print(core);
    process::exit(126)
}

pub fn is_binary(command_name: &str, core: &mut ShellCore) -> ! {
    InputError::BinaryFile(command_name.to_string()).print(core);
    process::exit(126)
}

pub fn internal(s: &str) -> ! {
    panic!("SUSH INTERNAL ERROR: {s}")
}

pub fn check_e_option(core: &mut ShellCore) {
    if core.db.exit_status != 0 && core.db.flags.contains("e") && !core.suspend_e_option {
        normal(core);
    }
}
