//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{proc_ctrl, ShellCore, Feeder};
use crate::elements::Pipe;
use crate::elements::command::Command;
use crate::elements::command::paren::ParenCommand;
use crate::elements::subword::Subword;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use nix::unistd;
use std::{thread, time};
use std::fs::File;
use std::io::{BufReader, BufRead, Error};
use std::os::fd::{FromRawFd, RawFd};
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Clone, Default)]
pub struct CommandSubstitutionOld {
    pub text: String,
    command: ParenCommand,
}

impl Subword for CommandSubstitutionOld {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut pipe = Pipe::new("|".to_string());
        pipe.set(-1, unistd::getpgrp());
        let pid = self.command.exec(core, &mut pipe)?;
        let result = self.read(pipe.recv, core);
        proc_ctrl::wait_pipeline(core, vec![pid], false, false);
        result?;
        Ok(())
    }
}

impl CommandSubstitutionOld {
    fn set_line(&mut self, line: Result<String, Error>) -> bool {
        if let Ok(ln) = line {
            self.text.push_str(&ln);
            self.text.push('\n');
            return true;
        }
        false
    }

    fn interrupted(&mut self, count: usize, core: &mut ShellCore) -> Result<(), ExecError> {
        if count%100 == 99 { //To receive Ctrl+C
            thread::sleep(time::Duration::from_millis(1));
        }
        match core.sigint.load(Relaxed) {
            true  => Err(ExecError::Interrupted),
            false => Ok(()),
        }
    }

    fn read(&mut self, fd: RawFd, core: &mut ShellCore) -> Result<(), ExecError> {
        let f = unsafe { File::from_raw_fd(fd) };
        let reader = BufReader::new(f);
        self.text.clear();
        for (i, line) in reader.lines().enumerate() {
            self.interrupted(i, core)?;
            if ! self.set_line(line) {
                break;
            }
        }

        self.text.pop();
        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("`") {
            return Ok(None);
        }

        let mut ans = Self::default();
        ans.text = feeder.consume(1);
        let mut esc = false;
        while esc || ! feeder.starts_with("`") {
            if feeder.is_empty() {
                feeder.feed_additional_line(core)?;
                continue;
            }

            let len = feeder.scanner_char();
            let c = feeder.consume(len);

            if esc && (c == "$" || c == "\\" || c == "`") {
                ans.text.pop();
            }

            ans.text += &c;

            if ! esc && c == "\\" {
                esc = true;
                continue;
            }

            esc = false;
        }

        ans.text += &feeder.consume(1);
        let mut paren = ans.text.clone();
        paren.remove(0);
        paren.insert(0, '(');
        paren.pop();
        paren.push(')');

        let mut f = Feeder::new(&paren);
        if let Some(s) = ParenCommand::parse(&mut f, core, false)? {
            ans.command = s;
            return Ok(Some(ans));
        }

        Ok(None)
    }
}
