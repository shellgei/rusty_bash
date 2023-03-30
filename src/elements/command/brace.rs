//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::Command;
use nix::unistd::Pid;
use std::os::unix::prelude::RawFd;
use crate::elements::script::Script;
use crate::elements::redirect::Redirect;
use crate::file_descs::*;
use std::process::exit;
use nix::unistd;

fn tail_check(s: &String) -> bool{
    for ch in s.chars().rev() {
        match ch {
            ' ' => continue,
            '\n' => return true,
            ';' => return true,
            '\t' => return true,
            _ => return false,
        }
    }
    false
}

#[derive(Debug)]
pub struct CommandBrace {
    pub script: Script,
    text: String,
    pid: Option<Pid>, 
    pub substitution_text: String,
    fds: FileDescs,
    group_leader: bool,
}

impl Command for CommandBrace {
    fn exec_elems(&mut self, core: &mut ShellCore) {
             self.script.exec(core);
             if ! self.fds.no_connection() {
                 exit(core.vars["?"].parse::<i32>().unwrap());
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

impl CommandBrace {
    pub fn new(script: Script) -> CommandBrace{
        CommandBrace {
            script: script,
            pid: None,
            text: "".to_string(),
            substitution_text: "".to_string(),
            fds: FileDescs::new(),
            group_leader: false,
        }
    }

    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<CommandBrace> {
        if ! text.starts_with("{") {
            return None;
        }

        core.nest.push("{".to_string());

        let mut backup = text.clone();
        let mut ans;
        let mut input_success;

        loop {
            text.consume(1);
            if let Some(s) = Script::parse(text, core) {
                if ! tail_check(&s.text){
                    text.rewind(backup);
                    core.nest.pop();
                    return None;
                }
    
                let text = "{".to_owned() + &s.text.clone() + "}";
                ans = CommandBrace::new(s);
                ans.text = text;
            }else{
                (backup, input_success) = text.rewind_feed_backup(&backup, core);
                if ! input_success {
                    eprintln!("ESC");
                    text.consume(text.len());
                    core.nest.pop();
                    return None;
                }
                continue;
            }
    
           // if text.len() == 0 || text.nth(0) != '}' {
            if ! text.starts_with("}") {
                (backup, input_success) = text.rewind_feed_backup(&backup, core);
                if ! input_success {
                    text.consume(text.len());
                    core.nest.pop();
                    return None;
                }
            }else{
                break;
            }
        }

        text.consume(1);

        loop {
            ans.text += &text.consume_blank();

            if let Some(r) = Redirect::parse(text, core){
                    ans.text += &r.text;
                    ans.fds.redirects.push(Box::new(r));
            }else{
                break;
            }
        }

        core.nest.pop();
        Some(ans)
    }
}
