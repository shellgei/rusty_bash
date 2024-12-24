//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::{File, OpenOptions};
use std::os::fd::{IntoRawFd, RawFd};
use std::io::Error;
use crate::elements::{Pipe, io};
use crate::elements::word::Word;
use crate::{Feeder, ShellCore};
use crate::utils::exit;
use nix::unistd;
use nix::unistd::{Pid, ForkResult};

#[derive(Debug, Clone, Default)]
pub struct Redirect {
    pub text: String,
    pub symbol: String,
    pub right: Word,
    pub left: String,
    left_fd: RawFd,
    left_backup: RawFd,
    extra_left_backup: RawFd, // &>, &>>用
    pub herepipe: Option<Pipe>,
}

impl Redirect {
    pub fn connect(&mut self, restore: bool, core: &mut ShellCore) -> bool {
        let args = match self.right.eval(core) {
            Some(v) => v,
            None => return false,
        };

        if args.len() != 1 && ! self.symbol.starts_with("<<") {
            eprintln!("sush: {}: ambiguous redirect", self.right.text);
            return false;
        }else{
            self.right.text = args[0].clone();
        }

        match self.symbol.as_str() {
            "<" => self.redirect_simple_input(restore),
            ">" => self.redirect_simple_output(restore),
            ">&" => self.redirect_output_fd(restore),
            ">>" => self.redirect_append(restore),
            "&>" => self.redirect_both_output(restore),
            "<<<" => self.redirect_herestring(restore),
            _ => exit::internal(" (Unknown redirect symbol)"),
        }
    }

    fn set_left_fd(&mut self, default_fd: RawFd) {
        self.left_fd = if self.left.len() == 0 {
            default_fd
        }else{
            self.left.parse().unwrap()
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
                    io::close(fd, &format!("sush(fatal): file does not close"));
                    self.left_fd = -1;
                }
                result
            },
            _  => {
                eprintln!("sush: {}: {}", &self.right.text, Error::last_os_error().kind());
                false
            },
        }
    }

    fn redirect_simple_input(&mut self, restore: bool) -> bool {
        self.set_left_fd(0);
        self.connect_to_file(File::open(&self.right.text), restore)
    }

    fn redirect_simple_output(&mut self, restore: bool) -> bool {
        self.set_left_fd(1);
        self.connect_to_file(File::create(&self.right.text), restore)
    }

    fn redirect_output_fd(&mut self, _: bool) -> bool {
        let fd = match self.right.text.parse::<RawFd>() {
            Ok(n) => n,
            _     => return false,
        };

        self.set_left_fd(1);
        io::share(fd, self.left_fd)
    }

    fn redirect_append(&mut self, restore: bool) -> bool {
        self.set_left_fd(1);
        self.connect_to_file(OpenOptions::new().create(true)
                .write(true).append(true).open(&self.right.text), restore)
    }

    fn redirect_both_output(&mut self, restore: bool) -> bool {
        self.left_fd = 1;
        if ! self.connect_to_file(File::create(&self.right.text), restore){
            return false;
        }

        if restore {
            self.extra_left_backup = io::backup(2);
        }
        io::share(1, 2);
        true
    }

    fn redirect_herestring(&mut self, restore: bool) -> bool {
        true
    }

    pub fn restore(&mut self) {
        if self.left_backup >= 0 && self.left_fd >= 0 {
            io::replace(self.left_backup, self.left_fd);
        }
        if self.extra_left_backup >= 0 {
            io::replace(self.extra_left_backup, 2);
        }
    }

    pub fn new() -> Redirect {
        Redirect {
            left_fd: -1,
            left_backup: -1,
            extra_left_backup: -1,
            ..Default::default()
        }
    }

    fn eat_symbol(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_redirect_symbol(core);
        if len == 0 {
            return false;
        }

        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();
        true
    }

    fn eat_right(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let blank_len = feeder.scanner_blank(core);
        ans.text += &feeder.consume(blank_len);

        let w = match Word::parse(feeder, core, false) {
            Some(w) => w,
            _       => return false,
        };

        ans.text += &w.text.clone();
        ans.right = w;
        true
    }

    fn eat_left(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_uint(core);
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

    pub fn set_herepipe(&mut self, core: &mut ShellCore) {
        if self.symbol != "<<<" {
            return;
        }

        let mut herepipe = Pipe::new("<<<".to_string());
        herepipe.set(-1, Pid::from_raw(0));
        self.herepipe = Some(herepipe);
        self.right.text = self.right.eval_for_case_word(core).unwrap();

    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Redirect> {
        let mut ans = Self::new();
        feeder.set_backup(); //追加

        if Self::eat_left(feeder, &mut ans, core) &&
           Self::eat_symbol(feeder, &mut ans, core) &&
           Self::eat_right(feeder, &mut ans, core) {
            feeder.pop_backup();
            Some(ans)
        }else{
            feeder.rewind(); //追加
            None
        }
    }
}
