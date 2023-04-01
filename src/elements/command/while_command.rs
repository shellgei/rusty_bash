//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::Command;
use std::os::unix::prelude::RawFd;
use crate::elements::script::Script;
use crate::elements::redirect::Redirect;
use nix::unistd::Pid;
use nix::unistd;
use crate::file_descs::*;

#[derive(Debug)]
pub struct CommandWhile {
    pub conddo: Option<(Script, Script)>,
    text: String,
    pid: Option<Pid>,
    fds: FileDescs,
    group_leader: bool,
}

impl Command for CommandWhile {
    fn get_pid(&self) -> Option<Pid> { self.pid }
    fn set_pid(&mut self, pid: Pid) { self.pid = Some(pid); }
    fn set_group(&mut self){
        if self.group_leader {
            let pid = nix::unistd::getpid();
            let _ = unistd::setpgid(pid, pid);
        }
    }
    fn set_group_leader(&mut self) { self.group_leader = true; }
    fn no_connection(&self) -> bool { self.fds.no_connection() }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
    }

    fn set_child_io(&mut self, core: &mut ShellCore) -> Result<(), String> {
        self.fds.set_child_io(core)
    }

    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }
    fn get_text(&self) -> String { self.text.clone() }

    fn exec_elems(&mut self, core: &mut ShellCore) {
        loop {
            if let Some((cond, doing)) = &mut self.conddo {
                cond.exec(core);
                if core.vars["?"] != "0" {
                    core.set_var("?", "0");
                    break;
                }
                doing.exec(core);
            }
        }
    }
}

impl CommandWhile {
    pub fn new() -> CommandWhile{
        CommandWhile {
            conddo: None,
            text: "".to_string(),
            fds: FileDescs::new(),
            pid: None,
            group_leader: false,
        }
    }


    fn eat_cond_do_pair(text: &mut Feeder, core: &mut ShellCore, ans: &mut CommandWhile) -> bool {
        core.nest.push("while".to_string());
        let cond = if let Some(s) = Script::parse(text, core) {
            ans.text += &s.text;
            s
        }else{
            core.nest.pop();
            return false;
        };

        core.nest.pop();
        ans.text += &text.consume(2); //always "do"
        core.nest.push("do".to_string());

        let doing = if let Some(s) = Script::parse(text, core) {
            ans.text += &s.text;
            s
        }else{
            core.nest.pop();
            return false;
        };

        ans.conddo = Some( (cond, doing) );
        core.nest.pop();
        true
    }

    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<CommandWhile> {
        if text.len() < 5 || ! text.starts_with("while") {
            return None;
        }

        let backup = text.clone();

        let mut ans = CommandWhile::new();
        ans.text += &text.consume(5);

        if ! CommandWhile::eat_cond_do_pair(text, core, &mut ans) {
            text.rewind(backup);
            return None;
        }

        ans.text += &text.consume(4); //always "done"

        while Redirect::eat_me(text, core, &mut ans.text, &mut ans.fds) {}
        Some(ans)
    }
}
