//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::env;

use nix::unistd::{execvpe, fork, ForkResult, Pid}; 
use std::ffi::CString;
use std::process::exit;
use std::os::unix::prelude::RawFd;

use crate::{ShellCore,Feeder};
use crate::abst_elems::CommandElem;
use crate::utils::*;

use crate::abst_elems::{PipelineElem, compound};
use crate::elem_arg::Arg;
use crate::elem_end_of_command::Eoc;
use crate::elem_redirect::Redirect;
use crate::elem_substitution::Substitution;
use crate::scanner::*;
use crate::utils_io::*;

/* command: delim arg delim arg delim arg ... eoc */
pub struct Command {
    vars: Vec<Box<Substitution>>,
    pub args: Vec<Box<dyn CommandElem>>,
    pub eoc: Option<Eoc>,
    pub text: String,
    pub pid: Option<Pid>,
    fds: FileDescs,
}

impl PipelineElem for Command {

    fn exec(&mut self, conf: &mut ShellCore) {
        if conf.has_flag('v') {
            eprintln!("{}", self.text.trim_end());
        }

        let mut args = self.eval(conf);

        if conf.has_flag('x') {
            eprintln!("+{}", args.join(" "));
        }

        // This sentence avoids an unnecessary fork for an internal command.
        if self.fds.no_connection() {
            if conf.functions.contains_key(&args[0]) {
                self.exec_function(&mut args, conf);
                return;
            }
            if self.run_on_this_process(&mut args, conf) {
                return;
            }
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    self.fds.set_child_io();
                    self.exec_external_command(&mut args, conf)
                },
                Ok(ForkResult::Parent { child } ) => {
                    self.pid = Some(child);
                    return;
                },
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }

    fn set_pipe(&mut self, pin: RawFd, pout: RawFd, pprev: RawFd) {
        self.fds.pipein = pin;
        self.fds.pipeout = pout;
        self.fds.prevpipein = pprev;
    }

    fn get_pid(&self) -> Option<Pid> { self.pid }
    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }

    fn get_eoc_string(&mut self) -> String {
        if let Some(e) = &self.eoc {
            return e.text.clone();
        }

        "".to_string()
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl Command {
    pub fn new() -> Command{
        Command {
            vars: vec!(),
            args: vec!(),
            eoc: None,
            text: "".to_string(),
            pid: None,
            fds: FileDescs::new(),
        }
    }

    fn run_on_this_process(&mut self, args: &mut Vec<String>, conf: &mut ShellCore) -> bool {
        if let Some(func) = conf.get_internal_command(&args[0]) {
            let status = func(conf, args);
            conf.vars.insert("?".to_string(), status.to_string());
            true
        }else{
            false
        }
    }

    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let mut args = vec!();

        for arg in &mut self.args {
            for s in &arg.eval(conf) {
                args.append(&mut eval_glob(&s.clone()));
            }
        };

        args.iter()
            .map(|a| Arg::remove_escape(&a))
            .collect()
    }

    pub fn push_vars(&mut self, s: Substitution){
        self.text += &s.get_text();
        self.vars.push(Box::new(s));
    }

    pub fn push_elems(&mut self, s: Box<dyn CommandElem>){
        self.text += &s.get_text();
        self.args.push(s);
    }

    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("command: '{}'", self.text));
        for elem in &self.args {
            ans.append(&mut elem.parse_info());
        };

        blue_strings(&ans)
    }

    fn exec_function(&mut self, args: &mut Vec<String>, conf: &mut ShellCore) {
        let text = conf.get_function(&args[0]).unwrap();

        let mut feeder = Feeder::new_with(text);
        if let Some(mut f) = compound(&mut feeder, conf) {
            let backup = conf.args.clone();
            conf.args = args.to_vec();
            conf.return_enable = true;
            f.exec(conf);
            self.pid = f.get_pid();
            conf.args = backup;
            conf.return_enable = false;
        }else{
            panic!("Shell internal error on function");
        };
    }

    fn exec_external_command(&mut self, args: &mut Vec<String>, conf: &mut ShellCore) {
        if conf.functions.contains_key(&args[0]) {
            self.exec_function(args, conf);
            exit(0);
        }

        if let Some(func) = conf.get_internal_command(&args[0]) {
            exit(func(conf, args));
        }

        let cargs: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        //if conf.flags.d {
        if conf.has_flag('d') {
            eprintln!("{}", self.parse_info().join("\n"));
        };

        for v in &mut self.vars {
            let key = (*v).name.text.clone();
            let value =  (*v).value.eval(conf).join(" ");
            env::set_var(key, value);
        }

        let envs: Vec<CString> = std::env::vars()
            .map(|v| format!("{}={}", v.0, v.1))
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        let _ = execvpe(&cargs[0], &*cargs, &envs);

        eprintln!("Command not found");
        exit(127);
    }

    fn replace_alias(text: &mut Feeder, conf: &mut ShellCore) {
        let compos = scanner_until_escape(text, 0, " \n");
        let com = text.from_to(0, compos);
        if let Some(alias) = conf.aliases.get(&com){
            text.replace(&com, alias);
        }
    }

    fn substitutions_and_redirects(text: &mut Feeder, conf: &mut ShellCore, ans: &mut Command) {
        loop {
            ans.text += &text.consume_blank();

            if let Some(r) = Redirect::parse(text){
                ans.text += &r.text;
                ans.fds.redirects.push(Box::new(r));
            }else if let Some(s) = Substitution::parse(text, conf) {
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

    fn args_and_redirects(text: &mut Feeder, conf: &mut ShellCore, ans: &mut Command) -> bool {
        let mut ok = false;
        loop {
            let backup = text.clone();
            if let Some(r) = Redirect::parse(text){
                ans.text += &r.text;
                ans.fds.redirects.push(Box::new(r));
            }else if let Some(a) = Arg::parse(text, conf, false, false) {
                if ! Command::ng_check(&a.text, ans.args.len() == 0){
                    text.rewind(backup);
                    break;
                }
                ans.push_elems(Box::new(a));
                ok = true;
            }else{
                break;
            }

            ans.text += &text.consume_blank();
    
            if text.len() == 0 {
                break;
            }

            if let Some(e) = Eoc::parse(text){
                ans.text += &e.text;
                ans.eoc = Some(e);
                break;
            }

            if scanner_end_paren(text, 0) == 1 || scanner_start_paren(text, 0) == 1 {
                break;
            }
        }

        ok
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Command> {
        let backup = text.clone();
        let mut ans = Command::new();

        if scanner_start_brace(text, 0) == 1 {
            return None;
        };

        Command::substitutions_and_redirects(text, conf, &mut ans);
        if conf.has_flag('i') {
            Command::replace_alias(text, conf);
        }

        if Command::args_and_redirects(text, conf, &mut ans) {
            Some(ans)
        }else{
            text.rewind(backup);
            None
        }
    }
}
