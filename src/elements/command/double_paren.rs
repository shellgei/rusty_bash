//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::Command;
use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use crate::elements::redirect::Redirect;
use crate::file_descs::*;
//use crate::feeder::scanner::*;
use crate::calculator::calculate;
use nix::unistd;

#[derive(Debug)]
pub struct CommandDoubleParen {
    text: String,
    expression: String,
    pid: Option<Pid>, 
    pub substitution_text: String,
    pub substitution: bool,
    fds: FileDescs,
    group_leader: bool,
}

impl Command for CommandDoubleParen {
    fn exec(&mut self, core: &mut ShellCore) {
        self.substitution_text = calculate(self.expression.clone(), core);

        let status = if self.substitution_text == "0" {
            "1"
        }else{
            "0"
        }.to_string();

        core.set_var("?", &status);
    }

    fn get_pid(&self) -> Option<Pid> { self.pid }
    fn set_group(&mut self){
        if self.group_leader {
            let pid = nix::unistd::getpid();
            let _ = unistd::setpgid(pid, pid);
        }
    }
    fn set_group_leader(&mut self) { self.group_leader = true; }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
    }

    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }
    fn get_text(&self) -> String { self.text.clone() }
}

impl CommandDoubleParen {
    pub fn new() -> CommandDoubleParen{
        CommandDoubleParen {
           // script: None,
            pid: None,
            text: "".to_string(),
            expression: "".to_string(),
            substitution_text: "".to_string(),
            substitution: false,
            fds: FileDescs::new(),
            group_leader: false,
        }
    }

    // TODO: this function must parse ((1+$(echo a | wc -l))) for example. 
    pub fn parse(text: &mut Feeder, core: &mut ShellCore, substitution: bool) -> Option<CommandDoubleParen> {
        if text.len() < 2 || ! text.starts_with( "((") {
            return None;
        }

        let mut backup = text.clone();
        let mut ans = CommandDoubleParen::new();
        let mut input_success;

        loop{
            ans.text = text.consume(2);

            let pos = text.scanner_until(0, ")");
            if text.len() > pos+1 && text.nth(pos) == ')' && text.nth(pos+1) != ')' {
                text.rewind(backup);
                return None;
            }

            if pos != text.len() {
                ans.expression = text.consume(pos);
                ans.text += &ans.expression.clone();
            }else{
                (backup, input_success) = text.rewind_feed_backup(&backup, core);
                if ! input_success {
                    text.consume(text.len());
                    return None;
                }
                continue;
            }

            if /*text.len() < 2 ||*/ ! text.starts_with( "))") {
                (backup, input_success) = text.rewind_feed_backup(&backup, core);
                if ! input_success {
                    text.consume(text.len());
                    return None;
                }
            }else{
                break;
            }
        }

        text.consume(2);
        if substitution {
            return Some(ans);
        }

        while Redirect::eat_me(text, core, &mut ans.text, &mut ans.fds) {}
        Some(ans)
    }
}
