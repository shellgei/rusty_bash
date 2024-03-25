//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::io;

pub struct Feeder {
    remaining: String,
}

impl Feeder {
    pub fn new() -> Feeder {
        Feeder {
            remaining: String::new(),
        }
    }

    pub fn feed_line(&mut self) {
        io::stdin().read_line(&mut self.remaining).expect("エラー");
        print!("{}", &self.remaining);
    } 
}
