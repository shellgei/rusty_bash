//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn shopt(core: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    core.shopts.print_all();
    0
}
