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

    fn read_line_stdin() -> String {
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0)  => panic!("おわり！"),
            Ok(_)  => line,
            Err(e) => panic!("sush: error reading input file: {}", &e),
        }
    }

    pub fn feed_line(&mut self) {
        self.remaining = Self::read_line_stdin();
        print!("{}", &self.remaining);
    } 
}
