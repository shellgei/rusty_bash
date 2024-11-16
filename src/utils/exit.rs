//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::process;

pub fn normal(core: &mut ShellCore) -> ! {
    core.write_history_to_file();

    let es_str = core.data.get_param("?");
    let exit_status = match es_str.parse::<i32>() {
        Ok(n)  => n%256,
        Err(_) => {
            eprintln!("exit: {}: numeric argument required", es_str);
            2
        },
    };

    process::exit(exit_status)
}
