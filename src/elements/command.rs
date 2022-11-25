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

use crate::abst_elems::{Compound, compound};
use crate::elements::arg::Arg;
use crate::elements::redirect::Redirect;
use crate::elements::substitution::Substitution;
use crate::scanner::*;
use crate::file_descs::*;

/* command: delim arg delim arg delim arg ... eoc */
pub struct Command {
    vars: Vec<Box<Substitution>>,
    pub args: Vec<Box<dyn CommandElem>>,
    pub text: String,
    pub pid: Option<Pid>,
    fds: FileDescs,
}

impl Compound for Command {
    fn exec(&mut self, core: &mut ShellCore) {
        if self.args.len() == 0 {
            self.set_vars(core);
            return;
        }

        if core.has_flag('v') {
            eprintln!("{}", self.text.trim_end());
        }

        let mut args = self.eval(core);
        core.set_var("_", &args[args.len()-1]);

        if core.has_flag('x') {
            eprintln!("+{}", args.join(" "));
        }

        // This sentence avoids an unnecessary fork for an internal command.
        if self.fds.no_connection() {
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
                if let Err(s) = self.fds.set_child_io(core){
                    eprintln!("{}", s);
                    exit(1);
                }
                self.exec_external_command(&mut args, core)
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

    fn get_pid(&self) -> Option<Pid> { self.pid }
    fn get_pipe_end(&mut self) -> RawFd { self.fds.pipein }
    fn get_pipe_out(&mut self) -> RawFd { self.fds.pipeout }
    fn get_text(&self) -> String { self.text.clone() }
}

impl Command {
    pub fn new() -> Command{
        Command {
            vars: vec![],
            args: vec![],
            //eoc: None,
            text: "".to_string(),
            pid: None,
            fds: FileDescs::new(),
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

        for arg in &mut self.args {
            for s in &arg.eval(core) {
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

    fn exec_function(&mut self, args: &mut Vec<String>, core: &mut ShellCore) {
        let text = core.get_function(&args[0]).unwrap();

        let mut feeder = Feeder::new_from(text);
        if let Some(mut f) = compound(&mut feeder, core) {
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
    }

    fn exec_external_command(&mut self, args: &mut Vec<String>, core: &mut ShellCore) {
        if core.functions.contains_key(&args[0]) {
            self.exec_function(args, core);
            exit(0);
        }

        if let Some(func) = core.get_builtin(&args[0]) {
            exit(func(core, args));
        }

        //let fullpath = get_fullpath(&args[0]);
        //args[0] = fullpath;
        args[0] = get_fullpath(&args[0]);

        let cargs: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        if core.has_flag('d') {
            eprintln!("{}", self.parse_info().join("\n"));
        };

        for v in &mut self.vars {
            let key = (*v).name.text.clone();
            let value =  (*v).value.eval(core).join(" ");
            env::set_var(key, value);
        }
        env::set_var("_".to_string(), args[0].clone());

        let envs: Vec<CString> = std::env::vars()
            .map(|v| format!("{}={}", v.0, v.1))
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        let _ = execvpe(&cargs[0], &cargs, &envs);

        eprintln!("Command not found");
        exit(127);
    }

    fn replace_alias(text: &mut Feeder, core: &mut ShellCore) {
        let compos = scanner_until_escape(text, 0, " \n");
        let com = text.from_to(0, compos);
        if let Some(alias) = core.aliases.get(&com){
            text.replace(&com, alias);
        }
    }

    fn substitutions_and_redirects(text: &mut Feeder, core: &mut ShellCore, ans: &mut Command) {
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

    fn args_and_redirects(text: &mut Feeder, core: &mut ShellCore, ans: &mut Command) -> bool {
        let mut ok = false;
        loop {
            let backup = text.clone();
            if let Some(r) = Redirect::parse(text, core){
                ans.text += &r.text;
                ans.fds.redirects.push(Box::new(r));
            }else if let Some(a) = Arg::parse(text, core, false, false) {
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

            let n = scanner_comment(text, 0);
            if n != 0 { 
                text.consume(n);
            }

            let (n, _) = scanner_control_op(text);
            if n != 0 { 
                break;
            }

            if text.starts_with("(") || text.starts_with(")") {
            //if scanner_end_paren(text, 0) == 1 || scanner_start_paren(text, 0) == 1 {
                break;
            }
        }

        ok
    }

    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<Command> {
        let backup = text.clone();
        let mut ans = Command::new();

        //if scanner_start_brace(text, 0) == 1 {
        if text.starts_with("{") {
            return None;
        };

        Command::substitutions_and_redirects(text, core, &mut ans);
        if core.has_flag('i') {
            Command::replace_alias(text, core);
        }

        if Command::args_and_redirects(text, core, &mut ans) || ans.vars.len() != 0 {
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
