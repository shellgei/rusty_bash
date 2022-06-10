//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use std::env;

use nix::unistd::{execvpe, fork, ForkResult, Pid, dup2}; 
use std::ffi::CString;
use std::process::exit;
use std::os::unix::prelude::RawFd;
use std::os::unix::io::IntoRawFd;
use std::fs::OpenOptions;

use crate::{ShellCore,Feeder,CommandElem};
use crate::utils::{blue_string, dup_and_close};

use crate::abst_hand_input_unit::HandInputUnit;
use crate::elem_arg::Arg;
use crate::elem_arg_delimiter::ArgDelimiter;
use crate::elem_end_of_command::Eoc;
use crate::elem_redirect::Redirect;
use crate::elem_substitution::Substitution;

/* command: delim arg delim arg delim arg ... eoc */
pub struct Command {
    vars: Vec<Box<Substitution>>,
    pub args: Vec<Box<dyn CommandElem>>,
    pub redirects: Vec<Box<Redirect>>,
    pub text: String,
    /* The followings are set by the pipeline element. */
    pub expansion: bool, 
    pub outfd_expansion: RawFd,
    pub infd_expansion: RawFd,
    pub pid: Option<Pid>,
}

impl HandInputUnit for Command {

    fn exec(&mut self, conf: &mut ShellCore) -> Option<Pid> {
        let mut args = self.eval(conf);

        if !self.expansion { // This sentence avoids an unnecessary fork for an internal command.
            if let Some(func) = conf.get_internal_command(&args[0]) {
                let status = func(conf, &mut args);
                conf.vars.insert("?".to_string(), status.to_string());
                return None
            }
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => self.exec_external_command(&mut args, conf),
                Ok(ForkResult::Parent { child } ) => return Some(child),
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }

        None
    }
}

impl Command {
    pub fn new() -> Command{
        Command {
            vars: vec!(),
            args: vec!(),
            redirects: vec!(),
            text: "".to_string(),
            expansion: false,
            outfd_expansion: 1,
            infd_expansion: 0,
            pid: None,
        }
    }

    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let mut args = vec!();

        for arg in &mut self.args {
            for s in &arg.eval(conf) {
                args.append(&mut Arg::expand_glob(&s.clone()));
            }
        };

        args.iter()
            .map(|a| Arg::remove_escape(&a))
            .collect()
    }

    fn set_redirect_fds(&self, r: &Box<Redirect>){
        if let Ok(num) = r.path[1..].parse::<i32>(){
            dup2(num, r.left_fd).expect("Invalid fd");
        }else{
            panic!("Invalid fd number");
        }
    }

    fn set_redirect(&self, r: &Box<Redirect>){
        if r.path.len() == 0 {
            panic!("Invalid redirect");
        }

        if r.direction_str == ">" {
            if r.path.chars().nth(0) == Some('&') {
                self.set_redirect_fds(r);
                return;
            }

            if let Ok(file) = OpenOptions::new().truncate(true).write(true).create(true).open(&r.path){
                dup_and_close(file.into_raw_fd(), r.left_fd);
            }else{
                panic!("Cannot open the file: {}", r.path);
            };
        }else if r.direction_str == "&>" {
            if let Ok(file) = OpenOptions::new().truncate(true).write(true).create(true).open(&r.path){
                dup_and_close(file.into_raw_fd(), 1);
                dup2(1, 2).expect("Redirection error on &>");
            }else{
                panic!("Cannot open the file: {}", r.path);
            };
        }else if r.direction_str == "<" {
            if let Ok(file) = OpenOptions::new().read(true).open(&r.path){
                dup_and_close(file.into_raw_fd(), r.left_fd);
            }else{
                panic!("Cannot open the file: {}", r.path);
            };
        }
    }

    fn set_child_io(&mut self) {
        if self.expansion { // the case of command expansion
            dup_and_close(self.outfd_expansion, 1);
        }

        for r in &self.redirects {
            self.set_redirect(r);
        };
    }

    pub fn push_vars(&mut self, s: Substitution){
        self.text += &s.text();
        self.vars.push(Box::new(s));
    }

    pub fn push_elems(&mut self, s: Box<dyn CommandElem>){
        self.text += &s.text();
        self.args.push(s);
    }

    pub fn return_if_valid(ans: Command, text: &mut Feeder, backup: Feeder) -> Option<Command> {
        if ans.args.len() > 0 {
              Some(ans)
        }else{
            text.rewind(backup);
            None
        }
    }

    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("command: '{}'", self.text));
        for elem in &self.args {
            ans.append(&mut elem.parse_info());
        };

        blue_string(&ans)
    }

    fn exec_external_command(&mut self, args: &mut Vec<String>, conf: &mut ShellCore) {
        self.set_child_io();

        if let Some(func) = conf.get_internal_command(&args[0]) {
            exit(func(conf, args));
        }

        let cargs: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        if conf.flags.d {
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

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Command> {
        let backup = text.clone();
        let mut ans = Command::new();
    
        //TODO: bash permits redirections here. 
    
        /* A command starts with substitutions. */
        while let Some(s) = Substitution::parse(text, conf) {
            ans.push_vars(s);
    
            if let Some(d) = ArgDelimiter::parse(text){
                ans.push_elems(Box::new(d));
            }
        }
    
        //TODO: bash permits redirections here. 
    
        /* Then one or more arguments exist. */
        let mut first = true;
        while let Some(a) = Arg::parse(text, true, conf) {
            if text.len() != 0 {
                if text.nth(0) == ')' || text.nth(0) == '(' {
                    text.error_occuring = true;
                    text.error_reason = "Unexpected token found".to_string();
                    text.rewind(backup);
                    return None;
                };
            };
            //check of alias
            if first {
                first = false;
                if let Some(alias) = conf.aliases.get(&a.text){
                    let mut sub_feeder = Feeder::new_with(alias.to_string());
                    while let Some(a) = Arg::parse(&mut sub_feeder, true, conf) {
                        ans.push_elems(Box::new(a));
                        if let Some(d) = ArgDelimiter::parse(&mut sub_feeder){
                            ans.push_elems(Box::new(d));
                        }
                    }
                }else{
                    ans.push_elems(Box::new(a));
                }
            }else{
                ans.push_elems(Box::new(a));
            };
    
            if let Some(d) = ArgDelimiter::parse(text){
                ans.push_elems(Box::new(d));
            }
    
            /* When a redirect is found. The command ends with redirects. */
            let mut exist = false;
            while let Some(r) = Redirect::parse(text){
                exist = true;
                ans.redirects.push(Box::new(r));
                if let Some(d) = ArgDelimiter::parse(text){
                    ans.push_elems(Box::new(d));
                }
            }
            if exist {
                break;
            }
    
            if text.len() == 0 {
                break;
            }
    
            if let Some(e) = Eoc::parse(text){
                ans.push_elems(Box::new(e));
                break;
            }
        }
    
        Command::return_if_valid(ans, text, backup)
    }
}
