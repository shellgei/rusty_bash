//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::Pipe;
use crate::elements::command::Command;
use crate::elements::command::paren::ParenCommand;
use crate::elements::subword::{Subword, SubwordType};
use nix::unistd;
use std::fs::File;
use std::io::{BufReader, Read};
use std::os::fd::{FromRawFd, RawFd};

#[derive(Debug, Clone)]
pub struct CommandSubstitution {
    pub text: String,
    command: Option<ParenCommand>,
}

impl Subword for CommandSubstitution {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        let c = match self.command.as_mut() {
            Some(c) => c, 
            None => { 
                self.text = "".to_string();
                return true;
            },
        };

        let mut pipe = Pipe::new("|".to_string());
        pipe.set(-1, unistd::getpgrp());
        let pid = c.exec(core, &mut pipe);
        let result = self.read(pipe.recv, core);
        core.wait_pipeline(vec![pid]);
        result
    }

    fn get_type(&self) -> SubwordType { SubwordType::CommandSubstitution }
}

impl CommandSubstitution {
    fn new() -> CommandSubstitution {
        CommandSubstitution {
            text: String::new(),
            command: None,
        }
    }

    fn read(&mut self, fd: RawFd, core: &mut ShellCore) -> bool {
        let f = unsafe { File::from_raw_fd(fd) };
        let mut reader = BufReader::new(f);
        self.text.clear();
        let _ = reader.read_to_string(&mut self.text);
        self.text.pop();
        true
    }

    fn eat_blank_line(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        let com_num = feeder.scanner_comment();
        ans.text += &feeder.consume(com_num);
        if feeder.starts_with("\n") {
            ans.text += &feeder.consume(1);
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("$(") {
            return None;
        }
        let mut ans = Self::new();
        
        while Self::eat_blank_line(feeder, &mut ans, core) {}
        
        if feeder.starts_with(")") {
            ans.text += &feeder.consume(1);
            return Some(ans);
        }

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
