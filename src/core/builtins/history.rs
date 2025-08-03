//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::utils::arg;
use crate::ShellCore;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn history_c(core: &mut ShellCore) -> i32 {
    core.rewritten_history.clear();
    core.history.clear();
    0
}

pub fn history(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = arg::dissolve_options(&args.to_vec());
    if arg::consume_option("-c", &mut args) {
        return history_c(core);
    }

    if args.len() > 1 {
        let msg = format!("{}: invalid option", &args[1]);
        return super::error_exit(1, "history", &msg, core);
    }

    let mut number = 1;

    let filename = core.db.get_param("HISTFILE").unwrap_or_default();
    if filename.is_empty() {
        return 0;
    }

    let file = match File::open(&filename) {
        Ok(f) => f,
        _ => return 0,
    };

    let f = BufReader::new(file);
    for line in f.lines() {
        println!("{:5} {}", number, &line.unwrap());
        number += 1;
    }

    for h in core.history.iter().rev() {
        println!("{:5} {}", number, &h);
        number += 1;
    }

    0
}
