//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

fn unset_var(core: &mut ShellCore, name: &str) -> i32 {
    core.data.unset(name);
    0
}

pub fn unset(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    match args[1].as_ref() {
        "-f" => 0,
        "-v" => 0,
        name => unset_var(core, name),
    }
}
