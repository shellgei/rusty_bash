//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    match core.data.position_parameters.pop() {
        None => panic!("SUSH INTERNAL ERROR: empty param stack"),
        _    => {},
    }
    core.data.position_parameters.push(args.to_vec());
    0
}
