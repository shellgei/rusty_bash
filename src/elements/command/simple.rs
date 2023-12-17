//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::env;

use nix::unistd::{execve, fork, ForkResult, Pid}; 
use nix::unistd;
use std::ffi::CString;
use std::process::exit;
use std::os::unix::prelude::RawFd;

use crate::{ShellCore,Feeder};
use crate::utils::*;

use crate::core::proc;
use crate::elements::command::Command;
use crate::elements::command;
use crate::elements::word::Word;
use crate::elements::redirect::Redirect;
use crate::elements::substitution::Substitution;
//use crate::feeder::scanner::*;
use crate::file_descs::*;

#[derive(Debug)]
pub struct SimpleCommand {
    vars: Vec<Substitution>,
    pub args: Vec<Word>,
    pub text: String,
    pub pid: Option<Pid>,
    fds: FileDescs,
    pub group_leader: bool,
}

fn is_reserve(s: &String) -> bool {
    s == "then" || s == "else" || s == "elif" || s == "fi" || s == "done" || s == "do" || s == ";;"
}


impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore) {
        if self.args.len() == 0 && self.fds.no_pipe() {
            self.set_vars(core);
        }

        if core.has_flag('v') {
            eprintln!("{}", self.text.trim_end());
        }

        let mut args = self.eval(core);
        //eprintln!("NUM:{} {:?}", args.len(), &args); 
        if args.len() == 0 {
            core.set_var("_", "");
        }else{
            core.set_var("_", &args[args.len()-1]);
        }

        if core.has_flag('x') {
            eprintln!("+{}", args.join(" "));
        }

        // This sentence avoids an unnecessary fork for an internal command.
        if self.fds.no_connection() && args.len() != 0 {
            if core.functions.contains_key(&args[0]) {
                self.exec_function(&mut args, core);
                return;
            }
            if self.run_on_this_process(&mut args, core) {
                return;
            }
        }

        match unsafe{fork()} {
            Ok(ForkResult::Child) => {
                proc::set_signals();
                self.set_group();
                if let Err(s) = self.fds.set_child_io(core){
                    eprintln!("{}", s);
                    exit(1);
                }
                if args.len() != 0 {
                    self.exec_external_command(&mut args, core)
                }else{
                    exit(0);
                }
            },
            Ok(ForkResult::Parent { child } ) => {
                self.pid = Some(child);
                return;
            },
            Err(err) => panic!("Failed to fork. {}", err),
        }
    }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
    }

    fn set_group_leader(&mut self) { self.group_leader = true; }

    fn get_pid(&self) -> Option<Pid> { self.pid }
    fn set_group(&mut self){
        if self.group_leader {
            let pid = nix::unistd::getpid();
            let _ = unistd::setpgid(pid, pid);
        }
    }
    /*
    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }
    */
    fn get_text(&self) -> String { self.text.clone() }
}

impl SimpleCommand {
    pub fn new() -> SimpleCommand{
        SimpleCommand {
            vars: vec![],
            args: vec![],
            //eoc: None,
            text: "".to_string(),
            pid: None,
            fds: FileDescs::new(),
            group_leader: false,
        }
    }

    fn run_on_this_process(&mut self, args: &mut Vec<String>, core: &mut ShellCore) -> bool {
        if let Some(func) = core.get_builtin(&args[0]) {
            let status = func(core, args);
            core.set_var("?", &status.to_string());
            true
        }else{
            false
        }
    }

    fn eval(&mut self, core: &mut ShellCore) -> Vec<String> {
        let mut args = vec![];

        for word in &mut self.args {
            for s in &word.eval(core) {
                args.append(&mut eval_glob(&s.clone()));
            }
        };

        args.iter()
            .map(|a| Word::remove_escape(&a))
            .collect()
    }

    pub fn push_vars(&mut self, s: Substitution){
        self.text += &s.get_text();
        self.vars.push(s);
    }

    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("command: '{}'", self.text));
        for elem in &self.args {
            ans.append(&mut elem.parse_info());
        };

        blue_strings(&ans)
    }

    fn exec_function(&mut self, args: &mut Vec<String>, core: &mut ShellCore) {
        let text = core.get_function(&args[0]).unwrap();

        let mut feeder = Feeder::new_from(text);
        //dbg!("IN '{:?}'", &feeder);
        if let Some(mut f) = command::parse(&mut feeder, core) {
         //   eprintln!("FUNCTION '{:?}'", f);
            let backup = core.args.clone();
            core.args = args.to_vec();
            core.return_enable = true;
            f.exec(core);
            self.pid = f.get_pid();
            core.args = backup;
            core.return_enable = false;
        }else{
            panic!("Shell internal error on function");
        };
        //eprintln!("OUT '{}'", feeder._text());
    }

    fn exec_external_command(&mut self, args: &mut Vec<String>, core: &mut ShellCore) {
        if core.functions.contains_key(&args[0]) {
            self.exec_function(args, core);
            exit(0);
        }

        if let Some(func) = core.get_builtin(&args[0]) {
            exit(func(core, args));
        }

        let org = args[0].clone();
        args[0] = get_fullpath(&org);
        if args[0].len() == 0 {
            eprintln!("Command not found: {:?}", &org);
            exit(127);
        }

        let cargs: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        if core.has_flag('d') {
            eprintln!("{}", self.parse_info().join("\n"));
        };

        for v in &mut self.vars {
            let key = (*v).name.clone();
            let value =  (*v).value.eval(core).join(" ");
            env::set_var(key, value);
        }
        env::set_var("_".to_string(), args[0].clone());

        let envs: Vec<CString> = std::env::vars()
            .map(|v| format!("{}={}", v.0, v.1))
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        let _ = execve(&cargs[0], &cargs, &envs);

        eprintln!("Command not found: {:?}", &cargs[0]);
        exit(127);
    }

    fn replace_alias(text: &mut Feeder, core: &mut ShellCore) {
        let compos = text.scanner_until_escape(" \n");
        let com = text.from_to(0, compos);
        if let Some(alias) = core.aliases.get(&com){
            text.replace(&com, alias);
        }
    }

    fn substitutions_and_redirects(text: &mut Feeder, core: &mut ShellCore, ans: &mut SimpleCommand) {
        loop {
            ans.text += &text.consume_blank();

            if let Some(r) = Redirect::parse(text, core){
                ans.text += &r.text;
                ans.fds.redirects.push(Box::new(r));
            }else if let Some(s) = Substitution::parse(text, core) {
                ans.push_vars(s);
            }else{
                break;
            }
        }
    }

    fn ng_check(text: &String, is_first: bool) -> bool {
        if ! is_first {
            return true; 
        }

        if Some('}') == text.chars().nth(0) {
            return ! is_first;
        }

        ! is_reserve(text)
    }

    fn args_and_redirects(text: &mut Feeder, core: &mut ShellCore, ans: &mut SimpleCommand) -> bool {
        let mut ok = false;
        loop {
            let backup = text.clone();
            if let Some(r) = Redirect::parse(text, core){
                ans.text += &r.text;
                ans.fds.redirects.push(Box::new(r));
            }else if let Some(a) = Word::parse(text, core, false) {
                if ! SimpleCommand::ng_check(&a.text, ans.args.len() == 0){
                    text.rewind(backup);
                    break;
                }
               // ans.push_elems(a);
                ans.text += &a.get_text();
                ans.args.push(a);
                ok = true;
            }else{
                break;
            }

            ans.text += &text.consume_blank();
    
            if text.len() == 0 {
                break;
            }

            let n = text.scanner_comment();
            if n != 0 { 
                text.consume(n);
            }

            let (n, _) = text.scanner_control_op();
            if n != 0 { 
                break;
            }

            if text.starts_with("(") || text.starts_with(")") {
                break;
            }
        }

        ok
    }

    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<SimpleCommand> {
        let backup = text.clone();
        let mut ans = SimpleCommand::new();

        if text.starts_with("{") {
            return None;
        };

        SimpleCommand::substitutions_and_redirects(text, core, &mut ans);
        if core.has_flag('i') {
            Self::replace_alias(text, core);
        }

        if Self::args_and_redirects(text, core, &mut ans) || ans.vars.len() != 0 {
            Some(ans)
        }else{
            text.rewind(backup);
            None
        }
    }

    fn set_vars(&mut self, core: &mut ShellCore){
        for e in &mut self.vars {
            let sub = e.eval(core);
            let (key, value) = (sub[0].clone(), sub[1].clone());
            if let Ok(_) = env::var(&key) {
                env::set_var(key, value);
            }else{
                core.set_var(&key, &value);
            };
        };
    }
}
