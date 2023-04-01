//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::Command;
use std::os::unix::prelude::RawFd;
use crate::elements::script::Script;
use crate::elements::redirect::Redirect;
use nix::unistd::Pid;
use crate::file_descs::*;
use nix::unistd;

#[derive(Debug)]
pub struct CommandIf {
    pub ifthen: Vec<(Script, Script)>,
    pub else_do: Option<Script>,
    text: String,
    pid: Option<Pid>,
    fds: FileDescs,
    group_leader: bool,
}

impl Command for CommandIf {
    fn exec_elems(&mut self, core: &mut ShellCore) {
        for pair in self.ifthen.iter_mut() {
             pair.0.exec(core);
             if core.vars["?"] != "0" {
                continue;
             }
             pair.1.exec(core);
             return;
        }

        if let Some(s) = &mut self.else_do {
            s.exec(core);
        }
    }

    fn set_pid(&mut self, pid: Pid) { self.pid = Some(pid); }
    fn set_group(&mut self){
        if self.group_leader {
            let pid = nix::unistd::getpid();
            let _ = unistd::setpgid(pid, pid);
        }
    }
    fn set_group_leader(&mut self) { self.group_leader = true; }
    fn no_connection(&self) -> bool { self.fds.no_connection() }

    fn set_child_io(&mut self, core: &mut ShellCore) -> Result<(), String> {
        self.fds.set_child_io(core)
    }

    fn get_pid(&self) -> Option<Pid> { self.pid }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
    }

    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }
    fn get_text(&self) -> String { self.text.clone() }
}

impl CommandIf {
    pub fn new() -> CommandIf{
        CommandIf {
            ifthen: vec![],
            else_do: None,
            fds: FileDescs::new(),
            text: "".to_string(),
            pid: None,
            group_leader: false,
        }
    }


    fn eat_if_then(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut CommandIf) -> bool {
        core.nest.push("if".to_string());
        let cond = if let Some(s) = Script::parse(feeder, core) {
            ans.text += &s.text;
            s
        }else{
            core.nest.pop();
            return false;
        };

        ans.text += &feeder.consume(4); //always "then"
        core.nest.pop();
        core.nest.push("then".to_string());

        let doing = if let Some(s) = Script::parse(feeder, core) {
            ans.text += &s.text;
            s
        }else{
            core.nest.pop();
            return false;
        };

        ans.ifthen.push( (cond, doing) );
        core.nest.pop();
        true
    }

    fn eat_else_fi(text: &mut Feeder, core: &mut ShellCore, ans: &mut CommandIf) -> bool {
        core.nest.push("else".to_string());
        let result = if let Some(s) = Script::parse(text, core) {
            ans.text += &s.text;
            ans.else_do = Some(s);
            ans.text += &text.consume(2); //always "fi"
            true 
        }else{
            false
        };

        core.nest.pop();
        result
    }

    fn eat_redirect(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut CommandIf) -> bool {
        ans.text += &feeder.consume_blank();

        if let Some(r) = Redirect::parse(feeder, core){
            ans.text += &r.text;
            ans.fds.redirects.push(Box::new(r));
            true
        }else{
            false
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<CommandIf> {
        //dbg!("if parse {:?}", &feeder);
        if feeder.len() < 2 || ! feeder.starts_with("if") {
            return None;
        }

        let mut ans = CommandIf::new();
        let backup = feeder.clone();
        ans.text += &feeder.consume(2);

        loop {
            if ! CommandIf::eat_if_then(feeder, core, &mut ans) {
                feeder.rewind(backup);
                return None;
            }

            if feeder.starts_with( "fi"){
                ans.text += &feeder.consume(2);
                break;
            }else if feeder.starts_with( "elif"){
                ans.text += &feeder.consume(4);
                continue;
            }else if feeder.starts_with("else"){
                ans.text += &feeder.consume(4);
                if ! CommandIf::eat_else_fi(feeder, core, &mut ans){
                    feeder.rewind(backup);
                    return None;
                }
                break;
            }

            feeder.rewind(backup);
            return None;
        }

        while Self::eat_redirect(feeder, core, &mut ans) {}

        Some(ans)
    }
}
