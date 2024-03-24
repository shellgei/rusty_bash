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

    fn read_line_stdin(&mut self) {
        let mut line = String::new();
        io::stdin().read_line(&mut line)
                   .expect("Failed to read line");

        self.remaining = line;
    }

    pub fn feed_line(&mut self) {
        self.read_line_stdin();
        println!("{}", self.remaining);
    }
}
