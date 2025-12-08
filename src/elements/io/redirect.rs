//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

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
            "<" => self.redirect_simple_input(restore, core),  // <
            ">" => self.redirect_simple_output(restore, core), // >
            ">&" => self.redirect_output_fd(restore, core),    // >&2
            "<&" => self.redirect_input_fd(restore, core),     // <&2
            ">>" => self.redirect_append(restore, core),
            "&>" => self.redirect_both_output(restore, core),
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
        core: &mut ShellCore,
    ) -> Result<(), ExecError> {
        if restore {
            self.left_backup = core.fds.backup(self.left_fd);
        }

        match file_open_result {
            Ok(file) => {
                let fd = file.into_raw_fd();
                let result = core.fds.replace(fd, self.left_fd);
                if !result {
                    core.fds.close(fd);
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

    fn redirect_simple_input(&mut self, restore: bool,
                             core: &mut ShellCore) -> Result<(), ExecError> {
        self.set_left_fd(0);
        self.connect_to_file(File::open(&self.right.text), restore, core)
    }

    fn redirect_simple_output(&mut self, restore: bool, core: &mut ShellCore) -> Result<(), ExecError> {
        self.set_left_fd(1);
        self.connect_to_file(File::create(&self.right.text), restore, core)
    }

    fn redirect_output_fd(&mut self, restore: bool, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.right.text == "-" {
            self.set_left_fd(1);
            core.fds.close(self.left_fd);
            return Ok(());
        }

        let right_fd = match self.right.text.parse::<RawFd>() {
            Ok(n) => n,
            _ => return Err(ExecError::AmbiguousRedirect(self.right.text.clone())),
        };
        self.set_left_fd(1);

        if restore {
            self.left_backup = core.fds.backup(self.left_fd);
        }

        core.fds.share(right_fd, self.left_fd)
    }

    fn redirect_input_fd(&mut self, restore: bool, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.right.text == "-" {
            self.set_left_fd(0);
            core.fds.close(self.left_fd);
            return Ok(());
        }

        let right_fd = match self.right.text.parse::<RawFd>() {
            Ok(n) => n,
            _ => return Err(ExecError::AmbiguousRedirect(self.right.text.clone())),
        };
        self.set_left_fd(0);

        if restore {
            self.left_backup = core.fds.backup(self.left_fd);
        }

        core.fds.share(right_fd, self.left_fd)
    }

    fn redirect_append(&mut self, restore: bool, core: &mut ShellCore) -> Result<(), ExecError> {
        self.set_left_fd(1);
        self.connect_to_file(
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.right.text),
            restore,
            core,
        )
    }

    fn redirect_both_output(&mut self, restore: bool, core: &mut ShellCore) -> Result<(), ExecError> {
        self.left_fd = 1;
        self.connect_to_file(File::create(&self.right.text), restore, core)?;

        if restore {
            self.extra_left_backup = core.fds.backup(2);
        }
        core.fds.share(1, 2)
    }

    fn redirect_heredocument(
        &mut self,
        core: &mut ShellCore,
        restore: bool,
    ) -> Result<(), ExecError> {
        self.left_fd = 0;
        let (recv, send) = core.fds.pipe();

        if restore {
            self.left_backup = core.fds.backup(0);
        }

        let right = self.right.make_unquoted_word().unwrap_or("".to_string());
        let quoted = right != self.right.text;

        let text = match quoted {
            false => self.here_data.eval_as_alter(core)?, // TODO: make it precise
            true => self.here_data.text.clone(),
        };

        match unsafe { unistd::fork()? } {
            ForkResult::Child => {
                core.fds.close(recv);
                let mut f = unsafe { File::from_raw_fd(send) };
                let _ = write!(&mut f, "{}", &text);
                f.flush().unwrap();
                core.fds.close(send);
                process::exit(0);
            }
            ForkResult::Parent { child: _ } => {
                core.fds.close(send);
                core.fds.replace(recv, 0);
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
        let (recv, send) = core.fds.pipe();

        if restore {
            self.left_backup = core.fds.backup(0);
        }

        let text = self.right.eval_as_herestring(core)?;

        match unsafe { unistd::fork()? } {
            ForkResult::Child => {
                core.fds.close(recv);
                let mut f = unsafe { File::from_raw_fd(send) };
                let _ = writeln!(&mut f, "{}", &text);
                f.flush().unwrap();
                core.fds.close(send);
                process::exit(0);
            }
            ForkResult::Parent { child: _ } => {
                core.fds.close(send);
                core.fds.replace(recv, 0);
            }
        }
        Ok(())
    }

    pub fn restore(&mut self, core: &mut ShellCore) {
        if self.left_backup >= 0 && self.left_fd >= 0 {
            if self.left_backup == self.left_fd {
                core.fds.close(self.left_fd);
            } else {
                core.fds.replace(self.left_backup, self.left_fd);
            }
        }
        if self.extra_left_backup >= 0 {
            core.fds.replace(self.extra_left_backup, 2);
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

    fn show_heredoc_warning(&self, lineno: usize, feeder_lineno: usize, core: &mut ShellCore) {
        let msg = format!("warning: here-document at line {} delimited by end-of-file (wanted `{}')", lineno, &self.right.text.replace("\\", ""));
        let _ = core.db.set_param("LINENO", &feeder_lineno.to_string(), None);
        error::print(&msg, core);
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

        let mut end_nest = end.clone();
        let mut back_quote = false;
        let end_return = end.clone() + "\n";
        match feeder.nest.last().unwrap().0.as_str() {
            "(" => end_nest += ")",
            "`" => {
                back_quote = true;
                end_nest += ")";
            },
            _ => end_nest += "\n",
        }

        if feeder.starts_with("\n") {
            feeder.consume(1);
        }

        loop {
            if feeder.is_empty() &&feeder.feed_additional_line(core).is_err() {
                self.show_heredoc_warning(lineno, feeder.lineno-1, core);
                break;
            }

            if remove_tab {
                let len = feeder.scanner_tabs();
                feeder.consume(len);
            }

            if feeder.starts_with(&end_nest) || feeder.starts_with(&end_return) {
                feeder.consume(end.len());
                if !back_quote && feeder.starts_with(")") {
                    self.show_heredoc_warning(lineno, feeder.lineno, core);
                }
                break;
            }else if feeder.starts_with(&(end.clone())) {
                feeder.consume(end.len());
                self.show_heredoc_warning(lineno, feeder.lineno, core);
                break;
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
