//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::{File, OpenOptions};
use std::os::fd::{IntoRawFd, RawFd};
use std::io::Error;
use crate::elements::io;
use crate::{Feeder, ShellCore};

#[derive(Debug)]
pub struct Redirect {
    pub text: String,
    pub symbol: String,
    pub right: String,
    pub left: String,
    left_fd: RawFd,
    left_backup: RawFd,
}

impl Redirect {
    pub fn connect(&mut self, restore: bool) -> bool {
        match self.symbol.as_str() {
            "<" => self.redirect_simple_input(restore),
            ">" => self.redirect_simple_output(restore),
            ">>" => self.redirect_append(restore),
            _ => panic!("SUSH INTERNAL ERROR (Unknown redirect symbol)"),
        }
    }

    fn set_left_fd(&mut self, default_fd: RawFd) {
        self.left_fd = if self.left.len() == 0 {
            default_fd
        }else{
            self.left.parse().expect("SUSHI INTERNAL ERROR")
        };
    }

    fn connect_to_file(&mut self, file_open_result: Result<File,Error>, restore: bool) -> bool {
        if restore {
            self.left_backup = io::backup(self.left_fd);
        }

        match file_open_result {
            Ok(file) => {
                let fd = file.into_raw_fd();
                let result = io::replace(fd, self.left_fd);
                if ! result {
                    io::close(fd, &format!("sush(fatal): file cannot be closed"));
                    self.left_fd = -1;
                }
                result
            },
            _  => {
                eprintln!("sush: {}: {}", &self.right, Error::last_os_error().kind());
                false
            },
        }
    }

    fn redirect_simple_input(&mut self, restore: bool) -> bool {
        self.set_left_fd(0);
        self.connect_to_file(File::open(&self.right), restore)
    }

    fn redirect_simple_output(&mut self, restore: bool) -> bool {
        self.set_left_fd(1);
        self.connect_to_file(File::create(&self.right), restore)
    }

    fn redirect_append(&mut self, restore: bool) -> bool {
        self.set_left_fd(1);
        self.connect_to_file(OpenOptions::new().create(true)
                .write(true).append(true).open(&self.right), restore)
    }

    pub fn restore(&mut self) {
        if self.left_backup >= 0 && self.left_fd >= 0 {
            io::replace(self.left_backup, self.left_fd);
        }
    }

    pub fn new() -> Redirect {
        Redirect {
            text: String::new(),
            symbol: String::new(),
            right: String::new(),
            left: String::new(),
            left_fd: -1,
            left_backup: -1,
        }
    }

    fn eat_symbol(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match feeder.scanner_redirect_symbol(core) {
            0 => false,
            n => {
                ans.symbol = feeder.consume(n);
                ans.text += &ans.symbol.clone();
                true
            },
        }
    }

    fn eat_right(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let blank_len = feeder.scanner_blank(core);
        ans.text += &feeder.consume(blank_len);

        match feeder.scanner_word(core) {
            0 => false,
            n => {
                ans.right = feeder.consume(n);
                ans.text += &ans.right.clone();
                true
            },
        }
    }

    fn eat_left(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_nonnegative_integer(core);
        if len == 0 {
            return true; //左側なし（文法上OK）
        }

        ans.left = feeder.consume(len);
        ans.text += &ans.left.clone();

        match ans.left.parse::<RawFd>() {
            Ok(_) => true,
            _     => false,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Redirect> {
        let mut ans = Self::new();
        let backup = feeder.clone(); //追加

        if Self::eat_left(feeder, &mut ans, core) &&   //追加
           Self::eat_symbol(feeder, &mut ans, core) && //ifを除去
           Self::eat_right(feeder, &mut ans, core) {
            Some(ans)
        }else{
            feeder.rewind(backup); //追加
            None
        }
    }
}
