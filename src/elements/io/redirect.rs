//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::{File, OpenOptions};
use std::os::fd::{IntoRawFd, RawFd};
use std::io::Error;
use crate::elements::io;
use crate::elements::word::Word;
use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;

#[derive(Debug, Clone, Default)]
pub struct Redirect {
    pub text: String,
    pub symbol: String,
    pub right: Word,
    pub left: String,
    left_fd: RawFd,
    left_backup: RawFd,
    extra_left_backup: RawFd, // &>, &>>用
}

impl Redirect {
    pub fn connect(&mut self, restore: bool, core: &mut ShellCore) -> Result<(), ExecError> {
        let args = self.right.eval(core)?;
        if args.len() != 1 { 
            return Err(ExecError::AmbiguousRedirect(self.right.text.clone()));
        }

        self.right.text = args[0].clone();

        match self.symbol.as_str() {
            "<" => self.redirect_simple_input(restore),
            ">" => self.redirect_simple_output(restore),
            ">>" => self.redirect_append(restore),
            "&>" => self.redirect_both_output(restore),
            _ => panic!("SUSH INTERNAL ERROR (Unknown redirect symbol)"),
        }
    }

    fn set_left_fd(&mut self, default_fd: RawFd) {
        self.left_fd = if self.left.is_empty() {
            default_fd
        }else{
            self.left.parse().expect("SUSHI INTERNAL ERROR (invalid FD)")
        };
    }

    fn connect_to_file(&mut self, file_open_result: Result<File,Error>, restore: bool) -> Result<(), ExecError> {
        if restore {
            self.left_backup = io::backup(self.left_fd);
        }   

        match file_open_result {
            Ok(file) => {
                let fd = file.into_raw_fd();
                let result = io::replace(fd, self.left_fd);
                if ! result {
                    io::close(fd, "sush(fatal): file does not close");
                    self.left_fd = -1; 
                    let msg = format!("{}: cannot replace", &fd);
                    return Err(ExecError::Other(msg));
                }
                Ok(())
            },
            _  => {
                let msg = format!("{}: {}", &self.right.text, Error::last_os_error().kind());
                Err(ExecError::Other(msg))
            },
        }
    }

    fn redirect_simple_input(&mut self, restore: bool) -> Result<(), ExecError> {
        self.set_left_fd(0);
        self.connect_to_file(File::open(&self.right.text), restore)
    }

    fn redirect_simple_output(&mut self, restore: bool) -> Result<(), ExecError> {
        self.set_left_fd(1);
        self.connect_to_file(File::create(&self.right.text), restore)
    }

    fn redirect_append(&mut self, restore: bool) -> Result<(), ExecError> {
        self.set_left_fd(1);
        self.connect_to_file(OpenOptions::new().create(true)
                .append(true).open(&self.right.text), restore)
    }

    fn redirect_both_output(&mut self, restore: bool) -> Result<(), ExecError> {
        self.left_fd = 1;
        self.connect_to_file(File::create(&self.right.text), restore)?;

        if restore {
            self.extra_left_backup = io::backup(2);
        }
        io::share(1, 2)
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
            right: Word::from(vec![]),
            left_fd: -1,
            left_backup: -1,
            extra_left_backup: -1,
            ..Default::default()
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

    fn eat_right(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        let blank_len = feeder.scanner_blank(core);
        ans.text += &feeder.consume(blank_len);

        let w = match Word::parse(feeder, core) {
            Ok(Some(w)) => w,
            Ok(None)    => return Ok(false),
            Err(e)      => {
                feeder.rewind();
                return Err(e);
            },
        };

        ans.text += &w.text.clone();
        ans.right = w;
        Ok(true)
    }

    fn eat_left(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_nonnegative_integer(core);
        if len == 0 {
            return true; //左側なし（文法上OK）
        }

        ans.left = feeder.consume(len);
        ans.text += &ans.left.clone();

        ans.left.parse::<RawFd>().is_ok()
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<Self>, ParseError> {
        let mut ans = Self::new();
        feeder.set_backup(); //追加

        if Self::eat_left(feeder, &mut ans, core) &&
           Self::eat_symbol(feeder, &mut ans, core) &&
           Self::eat_right(feeder, &mut ans, core)? {
            feeder.pop_backup();
            Ok(Some(ans))
        }else{
            feeder.rewind(); //追加
            Ok(None)
        }
    }
}
