//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command::Command;
use std::os::unix::prelude::RawFd;
//use crate::operators::ControlOperator;
use crate::elements::script::Script;
use crate::elements::redirect::Redirect;
use nix::unistd::Pid;
use nix::unistd;
use crate::file_descs::*;
//use crate::feeder::scanner::*;
use crate::elements::word::Word;
use crate::bash_glob::glob_match;
// use crate::elements::CommandElem;

#[derive(Debug)]
pub struct CommandCase {
    pub word: Word,
    pub conddo: Vec<(Vec<String>, Option<Script>)>,
    text: String,
    pid: Option<Pid>,
    fds: FileDescs,
    group_leader: bool,
}

impl Command for CommandCase {
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

    fn set_child_io(&mut self, core: &mut ShellCore)  -> Result<(), String> {
        self.fds.set_child_io(core)
    }

    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }
    fn get_text(&self) -> String { self.text.clone() }

    fn exec_elems(&mut self, core: &mut ShellCore) {
        let word_str = self.word.eval(core).join(" ");

        for (cond, doing) in &mut self.conddo {
            let mut flag = false;
            for c in cond {
                if glob_match(c, &word_str) {
                    if let Some(d) = doing {
                        d.exec(core);
                    }
                    flag = true;
                    break;
                }
            }
            if flag {
                break;
            }
        }
    }
}

impl CommandCase {
    pub fn new(word: Word) -> CommandCase{
        CommandCase {
            word: word, 
            conddo: vec![],
            text: "".to_string(),
            fds: FileDescs::new(),
            pid: None,
            group_leader: false,
        }
    }


    fn parse_cond_do_pair(text: &mut Feeder, core: &mut ShellCore, ans: &mut CommandCase) -> bool {
        let mut conds = vec![];
        ans.text += &text.request_next_line(core);

        loop {
            let pos = text.scanner_until_escape("|)");
            if pos == 0 || pos == text.len() {
                core.nest.pop();
                return false;
            }
            conds.push(text.consume(pos));
            ans.text += &conds.last().unwrap().clone();

            if text.starts_with(")") {
                break;
            }else{
                ans.text += &text.consume(1);
            }
        }
        core.nest.push("_)".to_string());
        ans.text += &text.consume(1);
        //ans.text += &text.request_next_line(core);

        let doing = if let Some(s) = Script::parse(text, core) {
            ans.text += &s.text;
            Some(s)
        }else{
            core.nest.pop();
            return false;
        };


        //ans.text += &text.request_next_line(core);

        if text.starts_with(";;") {
            ans.text += &text.consume(2);
        }
        ans.conddo.push( (conds, doing) );
        core.nest.pop();
        true
    }

    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<CommandCase> {
        if text.len() < 4 || ! text.starts_with("case") {
            return None;
        }

        let backup = text.clone();
        let ans_text = text.consume(4) + &text.consume_blank();

        let word = if let Some(a) = Word::parse(text, core, false) {
            a
        }else{
            text.rewind(backup);
            return None;
        };

        let mut ans = CommandCase::new(word);
        ans.text = ans_text;

        ans.text += &text.consume_blank();

        if text.len() >= 2 && text.starts_with("in") {
            ans.text += &text.consume(2);
        }else{
            text.rewind(backup);
            return None;
        }

        loop {
            ans.text += &text.consume_blank_return();
            ans.text += &text.request_next_line(core);
            ans.text += &text.consume_blank_return();

            if text.len() >= 4 && text.starts_with("esac") {
                ans.text += &text.consume(4);
                break;
            }

            if ! CommandCase::parse_cond_do_pair(text, core, &mut ans) {
                text.rewind(backup);
                return None;
            }
        }

        while Redirect::eat_me(text, core, &mut ans.text, &mut ans.fds) {}

        if ans.conddo.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
