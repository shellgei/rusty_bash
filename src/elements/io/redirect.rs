//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::fs::{File, OpenOptions};
use std::os::fd::{IntoRawFd, RawFd};
use std::io::Error;
use crate::{Feeder, ShellCore};
use crate::elements::io;
use crate::elements::word::Word;
use crate::error::exec::ExecError;
use crate::utils::exit;
use nix::unistd;
use nix::unistd::ForkResult;
use std::os::fd::FromRawFd;
use std::io::Write;
use std::process;

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
        if self.symbol == "<<<" {
            return self.redirect_herestring(core);
        }

        let args = self.right.eval(core)?;
        if args.len() != 1 {
            return Err(ExecError::AmbiguousRedirect(self.right.text.clone()));
        }

        self.right.text = args[0].clone();

        match self.symbol.as_str() {
            "<" => self.redirect_simple_input(restore),
            ">" => self.redirect_simple_output(restore),
            ">&" => self.redirect_output_fd(restore),
            ">>" => self.redirect_append(restore),
            "&>" => self.redirect_both_output(restore),
            _ => exit::internal(" (Unknown redirect symbol)"),
        }
    }

    fn set_left_fd(&mut self, default_fd: RawFd) {
        self.left_fd = match self.left.len() {
            0 => default_fd,
            _ => self.left.parse().unwrap(),
        }
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

    fn redirect_simple_input(&mut self, restore: bool) -> Result<(), ExecError> {
        self.set_left_fd(0);
        if ! self.connect_to_file(File::open(&self.right.text), restore) {
            return Err(ExecError::Other("file error".to_string()));
        }
        Ok(())
    }

    fn redirect_simple_output(&mut self, restore: bool) -> Result<(), ExecError> {
        self.set_left_fd(1);
        if ! self.connect_to_file(File::create(&self.right.text), restore) {
            return Err(ExecError::Other("file error".to_string()));
        }
        Ok(())
    }

    fn redirect_output_fd(&mut self, _: bool) -> Result<(), ExecError> {
        let fd = match self.right.text.parse::<RawFd>() {
            Ok(n) => n,
            _     => return Err(ExecError::AmbiguousRedirect(self.right.text.clone())),
        };

        self.set_left_fd(1);
        io::share(fd, self.left_fd)
    }

    fn redirect_append(&mut self, restore: bool) -> Result<(), ExecError> {
        self.set_left_fd(1);
        if ! self.connect_to_file(OpenOptions::new().create(true)
                .write(true).append(true).open(&self.right.text), restore) {
            return Err(ExecError::Other("file error".to_string()));
        }
        Ok(())
    }

    fn redirect_both_output(&mut self, restore: bool) -> Result<(), ExecError> {
        self.left_fd = 1;
        if ! self.connect_to_file(File::create(&self.right.text), restore){
            return Err(ExecError::Other("file error".to_string()));
        }

        if restore {
            self.extra_left_backup = io::backup(2);
        }
        io::share(1, 2);
        Ok(())
    }

    fn redirect_herestring(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let (r, s) = unistd::pipe().expect("Cannot open pipe");
        let recv = r.into_raw_fd();
        let send = s.into_raw_fd();

        let text = self.right.eval_for_case_word(core)
                       .unwrap_or("".to_string());

        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                io::close(recv, "herestring close error (child recv)");
                let mut f = unsafe { File::from_raw_fd(send) };
                let _ = write!(&mut f, "{}\n", &text);
                f.flush().unwrap();
                io::close(send, "herestring close error (child send)");
                process::exit(0);
            },
            Ok(ForkResult::Parent { child: _ } ) => {
                io::close(send, "herestring close error (parent send)");
                io::replace(recv, 0);
            },
            Err(err) => panic!("sush(fatal): Failed to fork. {}", err),
        }
        Ok(())
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
            Ok(Some(w)) => w,
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
