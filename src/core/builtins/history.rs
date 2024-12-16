//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub fn history(core: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    let mut number = 1;

    let filename = core.db.get_param("HISTFILE");
    if filename == "" {
        return 0;
    }

    let file = match File::open(&filename) {
        Ok(f) => f,
        _     => return 0,
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
