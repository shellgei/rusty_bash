//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::Pipe;
use crate::elements::command::Command;
use crate::elements::command::paren::ParenCommand;
use crate::elements::subword::Subword;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use nix::unistd;

#[derive(Debug, Clone, Default)]
pub struct ProcessSubstitution {
    pub text: String,
    command: ParenCommand,
    pub direction: char,
}

impl Subword for ProcessSubstitution {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.direction != '<' {
            return Err(ExecError::Other(">() is not supported yet".to_string()));
        }

        let mut pipe = Pipe::new("|".to_string());
        pipe.set(-1, unistd::getpgrp());
        let _ = self.command.exec(core, &mut pipe)?;
        self.text = "/dev/fd/".to_owned() + &pipe.recv.to_string();
        Ok(())
    }
}

impl ProcessSubstitution {
    /*
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

        if ! self.text.is_empty() {
            self.text.pop();
        }
        Ok(())
    }
    */

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("<(") && ! feeder.starts_with(">(") {
            return Ok(None);
        }
        let mut ans = ProcessSubstitution::default();
        ans.text = feeder.consume(1);
        ans.direction = ans.text.chars().nth(0).unwrap();

        if let Some(pc) = ParenCommand::parse(feeder, core, true)? {
            ans.text += &pc.get_text();
            ans.command = pc;
            return Ok(Some(ans));
        }

        Ok(None)
    }
}
