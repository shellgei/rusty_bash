//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::{execvpe, fork, ForkResult, Pid, dup2, read, close}; 
use std::ffi::CString;
use std::process::exit;
use std::env;
use std::os::unix::prelude::RawFd;
use std::os::unix::io::IntoRawFd;
use nix::unistd::pipe;
use crate::elem_end_of_command::Eoc;

use crate::{ShellCore,Feeder,CommandElem};
use crate::utils::blue_string;
use crate::elem_arg::Arg;
use crate::elem_arg_delimiter::ArgDelimiter;
use std::fs::OpenOptions;

use nix::sys::wait::*;

use crate::elem_substitution::Substitution;
use crate::elem_redirect::Redirect;
use crate::abst_hand_input_unit::HandInputUnit;


fn redirect_to_file(from: RawFd, to: RawFd){
    close(to).expect(&("Can't close fd: ".to_owned() + &to.to_string()));
    dup2(from, to).expect("Can't copy file descriptors");
    close(from).expect(&("Can't close fd: ".to_owned() + &from.to_string()));
}

/* command: delim arg delim arg delim arg ... eoc */
pub struct Command {
    vars: Vec<Box<Substitution>>,
    pub args: Vec<Box<dyn CommandElem>>,
    pub redirects: Vec<Box<Redirect>>,
    text: String,
    pub expansion: bool,
    pub outfd_expansion: RawFd,
    pub infd_expansion: RawFd,
}

impl HandInputUnit for Command {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        //self.set_io(conf);
        self.eval_args(conf)
    }

    fn exec(&mut self, conf: &mut ShellCore) -> String{
        let mut args = self.eval(conf);

        if self.expansion {
            let p = pipe().expect("Pipe cannot open");
            self.infd_expansion = p.0;
            self.outfd_expansion = p.1;
        }

        if !self.expansion {
            if let Some(func) = conf.get_internal_command(&args[0]) {
                let status = func(conf, &mut args);
                conf.vars.insert("?".to_string(), status.to_string());
                return "".to_string();
            }
        }

        let mut return_string = "".to_string();
        unsafe {
            match fork() {
                Ok(ForkResult::Child) => {
                    self.exec_external_command(&mut args, conf)
                },
                Ok(ForkResult::Parent { child } ) => {
                    return_string = self.wait_command(child, conf)
                },
                Err(err) => {
                    panic!("Failed to fork. {}", err)
                },
            }
        }

        if let Some(c) = return_string.chars().last() {
            if c == '\n' {
                return return_string[0..return_string.len()-1].to_string();
            }
        }

        return_string
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
        }
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
                redirect_to_file(file.into_raw_fd(), r.left_fd);
            }else{
                panic!("Cannot open the file: {}", r.path);
            };
        }else if r.direction_str == "&>" {
            if let Ok(file) = OpenOptions::new().truncate(true).write(true).create(true).open(&r.path){
                redirect_to_file(file.into_raw_fd(), 1);
                dup2(1, 2).expect("Redirection error on &>");
            }else{
                panic!("Cannot open the file: {}", r.path);
            };
        }else if r.direction_str == "<" {
            if let Ok(file) = OpenOptions::new().read(true).open(&r.path){
                redirect_to_file(file.into_raw_fd(), r.left_fd);
            }else{
                panic!("Cannot open the file: {}", r.path);
            };
        }
    }

    fn set_io(&mut self) {
        if self.expansion { // the case of command expansion
            redirect_to_file(self.outfd_expansion, 1);
        }

        for r in &self.redirects {
            self.set_redirect(r);
        };
    }

    fn eval_args(&mut self, conf: &mut ShellCore) -> Vec<String> {
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
        self.set_io();

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

    fn wait_command(&self, child: Pid, conf: &mut ShellCore) -> String {
        let mut ans = "".to_string();

        if self.expansion {
            let mut ch = [0;1000];
            while let Ok(n) = read(self.infd_expansion, &mut ch) {
                ans += &String::from_utf8(ch[..n].to_vec()).unwrap();
                if n < 1000 {
                    break;
                };
            };
        }

        match waitpid(child, None)
            .expect("Faild to wait child process.") {
            WaitStatus::Exited(pid, status) => {
                conf.vars.insert("?".to_string(), status.to_string());
                if status != 0 {
                    eprintln!("Pid: {:?}, Exit with {:?}", pid, status);
                }
            }
            WaitStatus::Signaled(pid, signal, _) => {
                conf.vars.insert("?".to_string(), (128+signal as i32).to_string());
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal)
            }
            _ => {
                eprintln!("Unknown error")
            }
        };

        ans
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
