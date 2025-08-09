//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::Pipe;
use crate::elements::command::Command;
use crate::elements::command::paren::ParenCommand;
use crate::elements::subword::Subword;
use crate::elements::word::WordMode;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use nix::unistd;

#[derive(Debug, Clone, Default)]
pub struct ProcessSubstitution {
    pub text: String,
    command: ParenCommand,
    pub direction: char,
    pipe: Option<Pipe>,
}

impl Subword for ProcessSubstitution {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if self.direction == '>' {
            return self.substitute_in(core);
        }

        let mut pipe = Pipe::new("|".to_string());
        pipe.set(-1, unistd::getpgrp());
        let pid = self.command.exec(core, &mut pipe)?.unwrap();
        core.proc_sub_pid.push(pid);
        self.text = "/dev/fd/".to_owned() + &pipe.recv.to_string();
        Ok(())
    }

    fn set_pipe(&mut self) {
        if self.direction == '>' {
            self.pipe = Some(Pipe::new(">()".to_string()));
            self.pipe.as_mut().unwrap().set(-1, unistd::getpgrp());
        }
    }
}

impl ProcessSubstitution {
    fn substitute_in(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let pipe = self.pipe.as_mut().unwrap();
        let pid = self.command.exec(core, pipe)?.unwrap();
        core.proc_sub_pid.push(pid);
        core.proc_sub_fd.push(pipe.proc_sub_send);
        self.text = "/dev/fd/".to_owned() + &pipe.proc_sub_send.to_string();

        Ok(())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore, mode: &Option<WordMode>)
    -> Result<Option<Self>, ParseError> {
        if let Some(WordMode::Arithmetic) = mode {
            return Ok(None);
        }

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
