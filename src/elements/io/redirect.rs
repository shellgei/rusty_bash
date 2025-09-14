//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::io;
use crate::elements::subword;
use crate::elements::subword::filler::FillerSubword;
use crate::elements::word::Word;
use crate::elements::word::WordMode;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::utils::{exit, file_check};
use crate::{error, Feeder, ShellCore};
use nix::unistd;
use nix::unistd::ForkResult;
use std::fs::{File, OpenOptions};
use std::io::Error;
use std::io::Write;
use std::os::fd::FromRawFd;
use std::os::fd::{IntoRawFd, RawFd};
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
    here_data: Word,
    pub called_as_heredoc: bool,
}

impl Redirect {
    pub fn connect(&mut self, restore: bool, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.symbol == "<<" || self.symbol == "<<-" {
            return self.redirect_heredocument(core, restore);
        }
        if self.symbol == "<<<" {
            return self.redirect_herestring(core, restore);
        }

        let args = self.right.eval(core)?;
        if args.len() != 1 {
            return Err(ExecError::AmbiguousRedirect(self.right.text.clone()));
        }

        if core.db.flags.contains('r') {
            match self.symbol.as_str() {
                ">" | ">|" | "<>" | ">&" | "&>" | ">>" => {
                    let msg = format!("{}: restricted: cannot redirect output", &args[0]);
                    return Err(ExecError::Other(msg));
                }
                _ => {}
            }
        }

        self.right.text = args[0].clone();

        if core.options.query("noclobber")
            && (self.symbol.as_str() == ">" || self.symbol.as_str() == ">>")
            && file_check::exists(&self.right.text)
        {
            return Err(ExecError::CannotOverwriteExistingFile(
                self.right.text.clone(),
            ));
        }

        match self.symbol.as_str() {
            "<" => self.redirect_simple_input(restore),  // <
            ">" => self.redirect_simple_output(restore), // >
            ">&" => self.redirect_output_fd(restore),    // >&2
            "<&" => self.redirect_input_fd(restore),     // <&2
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

    fn connect_to_file(
        &mut self,
        file_open_result: Result<File, Error>,
        restore: bool,
    ) -> Result<(), ExecError> {
        if restore {
            self.left_backup = io::backup(self.left_fd);
        }

        match file_open_result {
            Ok(file) => {
                let fd = file.into_raw_fd();
                let result = io::replace(fd, self.left_fd);
                if !result {
                    io::close(fd, "sush(fatal): file does not close");
                    self.left_fd = -1;
                    let msg = format!("{}: cannot replace", &fd);
                    return Err(ExecError::Other(msg));
                }
                Ok(())
            }
            _ => {
                let msg = format!("{}: {}", &self.right.text, Error::last_os_error().kind());
                Err(ExecError::Other(msg))
            }
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

    fn redirect_output_fd(&mut self, restore: bool) -> Result<(), ExecError> {
        if self.right.text == "-" {
            self.set_left_fd(1);
            io::close(self.left_fd, "cannot close");
            return Ok(());
        }

        let right_fd = match self.right.text.parse::<RawFd>() {
            Ok(n) => n,
            _ => return Err(ExecError::AmbiguousRedirect(self.right.text.clone())),
        };
        self.set_left_fd(1);

        if restore {
            self.left_backup = io::backup(self.left_fd);
        }

        io::share(right_fd, self.left_fd)
    }

    fn redirect_input_fd(&mut self, restore: bool) -> Result<(), ExecError> {
        if self.right.text == "-" {
            self.set_left_fd(0);
            io::close(self.left_fd, "cannot close");
            return Ok(());
        }

        let right_fd = match self.right.text.parse::<RawFd>() {
            Ok(n) => n,
            _ => return Err(ExecError::AmbiguousRedirect(self.right.text.clone())),
        };
        self.set_left_fd(0);

        if restore {
            self.left_backup = io::backup(self.left_fd);
        }

        io::share(right_fd, self.left_fd)
    }

    fn redirect_append(&mut self, restore: bool) -> Result<(), ExecError> {
        self.set_left_fd(1);
        self.connect_to_file(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.right.text),
            restore,
        )
    }

    fn redirect_both_output(&mut self, restore: bool) -> Result<(), ExecError> {
        self.left_fd = 1;
        self.connect_to_file(File::create(&self.right.text), restore)?;

        if restore {
            self.extra_left_backup = io::backup(2);
        }
        io::share(1, 2)
    }

    fn redirect_heredocument(
        &mut self,
        core: &mut ShellCore,
        restore: bool,
    ) -> Result<(), ExecError> {
        self.left_fd = 0;
        let (r, s) = unistd::pipe().expect("Cannot open pipe");
        let recv = r.into_raw_fd();
        let send = s.into_raw_fd();

        if restore {
            self.left_backup = io::backup(0);
        }

        let right = self.right.make_unquoted_word().unwrap_or("".to_string());
        let quoted = right != self.right.text;

        let text = match quoted {
            false => self.here_data.eval_as_alter(core)?, // TODO: make it precise
            true => self.here_data.text.clone(),
        };

        match unsafe { unistd::fork()? } {
            ForkResult::Child => {
                io::close(recv, "here_data close error (child recv)");
                let mut f = unsafe { File::from_raw_fd(send) };
                let _ = write!(&mut f, "{}", &text);
                f.flush().unwrap();
                io::close(send, "here_data close error (child send)");
                process::exit(0);
            }
            ForkResult::Parent { child: _ } => {
                io::close(send, "here_data close error (parent send)");
                io::replace(recv, 0);
            }
        }
        Ok(())
    }

    fn redirect_herestring(
        &mut self,
        core: &mut ShellCore,
        restore: bool,
    ) -> Result<(), ExecError> {
        self.left_fd = 0;
        let (r, s) = unistd::pipe().expect("Cannot open pipe");
        let recv = r.into_raw_fd();
        let send = s.into_raw_fd();

        if restore {
            self.left_backup = io::backup(0);
        }

        let text = self.right.eval_as_herestring(core)?;

        match unsafe { unistd::fork()? } {
            ForkResult::Child => {
                io::close(recv, "here_data close error (child recv)");
                let mut f = unsafe { File::from_raw_fd(send) };
                let _ = writeln!(&mut f, "{}", &text);
                f.flush().unwrap();
                io::close(send, "here_data close error (child send)");
                process::exit(0);
            }
            ForkResult::Parent { child: _ } => {
                io::close(send, "here_data close error (parent send)");
                io::replace(recv, 0);
            }
        }
        Ok(())
    }

    pub fn restore(&mut self) {
        if self.left_backup >= 0 && self.left_fd >= 0 {
            if self.left_backup == self.left_fd {
                io::close(self.left_fd, "cannot close");
            } else {
                io::replace(self.left_backup, self.left_fd);
            }
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

    pub fn eat_heredoc(
        &mut self,
        feeder: &mut Feeder,
        core: &mut ShellCore,
        lineno: usize,
    ) -> Result<(), ParseError> {
        let remove_tab = self.symbol == "<<-";
        let end = match self.right.eval_as_value(core) {
            Ok(s) => s,
            Err(_) => return Err(ParseError::UnexpectedSymbol(self.right.text.clone())),
        };

        let end_return = end.clone() + "\n";

        if feeder.starts_with("\n") {
            feeder.consume(1);
        }

        loop {
            if feeder.is_empty() {
                if feeder.feed_additional_line(core).is_err() {
                    let msg = format!("warning: here-document at line {} delimited by end-of-file (wanted `{}')", lineno, &self.right.text);
                    let _ = core.db.set_param("LINENO", &feeder.lineno.to_string(), None);
                    error::print(&msg, core);
                    break;
                }

                if remove_tab {
                    let len = feeder.scanner_tabs();
                    feeder.consume(len);
                }

                if feeder.starts_with(&end_return) {
                    feeder.consume(end.len());
                    break;
                }else if feeder.starts_with(&end) {
                    feeder.consume(end.len());
                    let msg = format!("warning: here-document at line {} delimited by end-of-file (wanted `{}')", lineno, &self.right.text);
                    //let _ = core.db.set_param("LINENO", &lineno2.to_string(), None);
                    let _ = core.db.set_param("LINENO", &feeder.lineno.to_string(), None);
                    error::print(&msg, core);
                    break;
                }
            }

            if let Some(mut sw) = subword::parse(feeder, core, &Some(WordMode::Heredoc))? {
                sw.set_heredoc_flag();
                self.here_data.text += sw.get_text();
                self.here_data.subwords.push(sw);
            } else {
                let len = feeder.scanner_char();
                if len > 0 {
                    let c = feeder.consume(len);
                    self.here_data.text += &c;
                    self.here_data
                        .subwords
                        .push(Box::new(FillerSubword { text: c }));
                }
            }
        }

        Ok(())
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

        let w = match Word::parse(feeder, core, None) {
            Ok(Some(w)) => w,
            _ => return false,
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

        ans.left.parse::<RawFd>().is_ok()
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Redirect> {
        let mut ans = Self::new();
        feeder.set_backup(); //追加

        if Self::eat_left(feeder, &mut ans, core)
            && Self::eat_symbol(feeder, &mut ans, core)
            && Self::eat_right(feeder, &mut ans, core)
        {
            feeder.pop_backup();
            Some(ans)
        } else {
            feeder.rewind(); //追加
            None
        }
    }
}
