//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;

pub fn caller(core: &mut ShellCore, _: &[String]) -> i32 {
    let linenos_len = core.db.index_based_len("BASH_LINENO");
    if linenos_len == 0 {
        return 1;
    }
    let functions_len = core.db.index_based_len("FUNCNAME");

    let lineno = core.db.get_elem("BASH_LINENO", "0").unwrap();
    let mut funcname = core.db.get_elem("FUNCNAME", "1").unwrap();

    if funcname == "" {
        funcname = "NULL".to_string();
    }else {
        funcname = "main".to_string();
    }

    println!("{} {}", &lineno, funcname);
    0
}
