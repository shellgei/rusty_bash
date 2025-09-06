//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder, proc_ctrl};
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

#[derive(Debug, Clone)]
pub struct CommandSubstitution {
    pub text: String,
    command: ParenCommand,
}

impl Subword for CommandSubstitution {
    fn get_text(&self) -> &str {self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let mut pipe = Pipe::new("|".to_string());
        pipe.set(-1, unistd::getpgrp());
        let pid = self.command.exec(core, &mut pipe)?;
        let result = self.read(pipe.recv, core);
        proc_ctrl::wait_pipeline(core, vec![pid]);
        result
    }
}

impl CommandSubstitution {
    fn set_line(&mut self, line: Result<String, Error>) -> bool {
        if let Ok(ln) = line {
            self.text.push_str(&ln);
            self.text.push('\n');
            return true;
        }
        false
    }

    fn interrupted(&mut self, count: usize, core: &mut ShellCore)
                                         -> Result<(), ExecError> {
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

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
                         -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("$(") {
            return Ok(None);
        }
        let mut text = feeder.consume(1);

        if let Some(pc) = ParenCommand::parse(feeder, core, true)? {
            text += &pc.get_text();
            Ok(Some(Self {text, command: pc} ))
        }else{
            Ok(None)
        }
    }
}
