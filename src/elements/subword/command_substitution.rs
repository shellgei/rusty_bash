//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::Pipe;
use crate::elements::command::Command;
use crate::elements::command::paren::ParenCommand;
use crate::elements::subword::{Subword, SubwordType};
use nix::unistd;
use std::io::Read;
use std::fs::File;
use std::os::fd::{FromRawFd, RawFd};

#[derive(Debug, Clone)]
pub struct CommandSubstitution {
    pub text: String,
    command: Option<ParenCommand>,
}

impl Subword for CommandSubstitution {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitution(&mut self, core: &mut ShellCore) -> bool {
        match self.command {
            Some(_) => self.exec(core),
            _       => false,
        }
    }

    fn get_type(&self) -> SubwordType { SubwordType::CommandSubstitution }
    fn clear(&mut self) { self.text = String::new(); }
}

impl CommandSubstitution {
    fn new() -> CommandSubstitution {
        CommandSubstitution {
            text: String::new(),
            command: None,
        }
    }

    fn exec(&mut self, core: &mut ShellCore) -> bool {
        let c = self.command.as_mut().unwrap();
        let mut pipe = Pipe::new("|".to_string());
        pipe.set(-1, unistd::getpgrp());
        let pid = c.exec(core, &mut pipe);
        self.read(pipe.recv); 
        core.wait_pipeline(vec![pid]);
        true
    }

    fn read(&mut self, fd: RawFd) {
        let mut f = unsafe { File::from_raw_fd(fd) };
        self.text.clear();
        f.read_to_string(&mut self.text)
            .expect("sush: command substitution I/O error");
        if self.text.ends_with("\n") {
            self.text.pop();
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("$(") {
            return None;
        }
        let mut ans = CommandSubstitution::new();
        ans.text = feeder.consume(1);

        if let Some(pc) = ParenCommand::parse(feeder, core) {
            ans.text += &pc.get_text();
            ans.command = Some(pc);
            Some(ans)
        }else{
            None
        }
    }
}
