//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::{File, OpenOptions};
use std::os::fd::{RawFd, IntoRawFd};
use crate::elements::io;
use crate::{Feeder, ShellCore};

#[derive(Debug)]
pub struct Redirect {
    pub text: String,
    pub symbol: String,
    pub right: String,
    pub backup: RawFd,
    pub to_be_restored: RawFd,
}

impl Redirect {
    pub fn connect(&mut self, restore: bool) {
        match self.symbol.as_str() {
            "<" => self.redirect_simple_output(restore),
            ">" => self.redirect_simple_input(restore),
            ">>" => self.redirect_append(restore),
            _ => panic!("SUSH INTERNAL ERROR (Unknown redirect symbol)"),
        }
    }

    pub fn redirect_simple_output(&mut self, restore: bool) {
        if restore {
            self.to_be_restored = 0;
            self.backup = io::backup(0);
        }
        let fd = File::open(&self.right).unwrap().into_raw_fd();
        io::replace(fd, 0);
    }

    pub fn redirect_append(&mut self, restore: bool) {
        if restore {
            self.to_be_restored = 1;
            self.backup = io::backup(1);
        }
        let fd = OpenOptions::new().create(true).write(true).append(true)
                 .open(&self.right).unwrap().into_raw_fd();
        io::replace(fd, 1);
    }

    pub fn redirect_simple_input(&mut self, restore: bool) {
        if restore {
            self.to_be_restored = 1;
            self.backup = io::backup(1);
        }
        let fd = File::create(&self.right).unwrap().into_raw_fd();
        io::replace(fd, 1);
    }

    pub fn restore(&mut self) {
        io::replace(self.backup, self.to_be_restored);
    }

    pub fn new() -> Redirect {
        Redirect {
            text: String::new(),
            symbol: String::new(),
            right: String::new(),
            backup: -1,
            to_be_restored: -1,
        }
    }

    fn eat_symbol(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_redirect_symbol(core);
        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();
        len != 0
    }

    fn eat_right(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let blank_len = feeder.scanner_blank(core);
        ans.text += &feeder.consume(blank_len);

        let len = feeder.scanner_word(core);
        ans.right = feeder.consume(len);
        ans.text += &ans.right.clone();
        len != 0
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Redirect> {
        let mut ans = Self::new();

        if Self::eat_symbol(feeder, &mut ans, core) &&
           Self::eat_right(feeder, &mut ans, core) {
            Some(ans)
        }else{
            None
        }
    }
}
