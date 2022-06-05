//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::{execvpe, fork, ForkResult, Pid, dup2, read, close}; 
use nix::sys::wait::*;
use std::ffi::CString;
use std::process::exit;
use std::env;
use std::os::unix::prelude::RawFd;
use std::os::unix::io::IntoRawFd;

use crate::{ShellCore,Feeder,CommandPart};
use crate::utils::blue_string;
use crate::elems_in_command::{Arg, Substitution, Redirect};
use std::fs::OpenOptions;

pub trait Executable {
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> { vec!() }
    fn exec(&mut self, _conf: &mut ShellCore) -> String { "".to_string() }
}

pub struct BlankPart {
    pub elems: Vec<Box<dyn CommandPart>>,
    text: String,
}

impl Executable for BlankPart {
}

impl BlankPart {
    pub fn new() -> BlankPart{
        BlankPart {
            elems: vec!(),
            text: "".to_string(),
        }
    }

    pub fn push(&mut self, s: Box<dyn CommandPart>){
        self.text += &s.text();
        self.elems.push(s);
    }

    pub fn return_if_valid(ans: BlankPart) -> Option<BlankPart> {
        if ans.elems.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}

pub struct Substitutions {
    pub elems: Vec<Box<dyn CommandPart>>,
    text: String,
}

impl Substitutions {
    pub fn new() -> Substitutions{
        Substitutions {
            elems: vec!(),
            text: "".to_string(),
        }
    }

    pub fn return_if_valid(ans: Substitutions) -> Option<Substitutions> {
        if ans.elems.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}

impl Executable for Substitutions {
    fn exec(&mut self, conf: &mut ShellCore) -> String {
        if conf.flags.d {
            eprintln!("{}", self.parse_info().join("\n"));
        };

        for e in &mut self.elems {
            let sub = e.eval(conf);
            if sub.len() != 2{
                continue;
            };

            let (key, value) = (sub[0].clone(), sub[1].clone());
            if let Ok(_) = env::var(&key) {
                env::set_var(key, value);
            }else{
                conf.vars.insert(key, value);
            };
        };

        "".to_string()
    }
}

impl Substitutions {
    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("substitutions: '{}'", self.text));
        for elem in &self.elems {
            ans.append(&mut elem.parse_info());
        };
        
        blue_string(&ans)
    }

    pub fn push(&mut self, s: Box<dyn CommandPart>){
        self.text += &s.text();
        self.elems.push(s);
    }
}


/* command: delim arg delim arg delim arg ... eoc */
pub struct CommandWithArgs {
    vars: Vec<Box<Substitution>>,
    pub args: Vec<Box<dyn CommandPart>>,
    pub redirects: Vec<Box<Redirect>>,
    text: String,
    pub expansion: bool,
    pub pipe_outfd: RawFd,
    pub pipe_infd: RawFd,
}

impl Executable for CommandWithArgs {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        //self.set_io(conf);
        self.eval_args(conf)
    }

    fn exec(&mut self, conf: &mut ShellCore) -> String{
        let mut args = self.eval(conf);

        if !self.expansion {
            if let Some(func) = conf.get_internal_command(&args[0]) {
                let _status = func(conf, &mut args);
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
                    return_string = self.wait_command(child)
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

fn redirect(from: RawFd, to: RawFd){
    close(to).expect(&("Can't close fd: ".to_owned() + &to.to_string()));
    dup2(from, to).expect("Can't copy file descriptors");
    close(from).expect(&("Can't close fd: ".to_owned() + &from.to_string()));
}

impl CommandWithArgs {
    pub fn new() -> CommandWithArgs{
        CommandWithArgs {
            vars: vec!(),
            args: vec!(),
            redirects: vec!(),
            text: "".to_string(),
            expansion: false,
            pipe_outfd: 1,
            pipe_infd: 0,
        }
    }

    fn set_file_io(&self, r: &Box<Redirect>){
        if r.direction_str == ">" {
            if let Ok(file) = OpenOptions::new().truncate(true).write(true).create(true).open(&r.path){
                //self.outfd = file.into_raw_fd();
                redirect(file.into_raw_fd(), r.left_fd);
            }else{
                panic!("Cannot open the file: {}", r.path);
            };
        }else if r.direction_str == "&>" {
            if let Ok(file) = OpenOptions::new().truncate(true).write(true).create(true).open(&r.path){
                close(1);
                dup2(file.into_raw_fd(), 1);
                close(2);
                dup2(1, 2);
            }else{
                panic!("Cannot open the file: {}", r.path);
            };
        }else if r.direction_str == "<" {
            if let Ok(file) = OpenOptions::new().read(true).open(&r.path){
                redirect(file.into_raw_fd(), r.left_fd);
            }else{
                panic!("Cannot open the file: {}", r.path);
            };
        }
    }

    fn set_io(&mut self) {
        if self.expansion { // the case of command expansion
            redirect(self.pipe_outfd, 1);
        }

        for r in &self.redirects {
            self.set_file_io(r);
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

    pub fn push_elems(&mut self, s: Box<dyn CommandPart>){
        self.text += &s.text();
        self.args.push(s);
    }

    pub fn return_if_valid(ans: CommandWithArgs, text: &mut Feeder, backup: Feeder) -> Option<CommandWithArgs> {
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

    fn wait_command(&self, child: Pid) -> String {
        let mut ans = "".to_string();

        if self.expansion {
            let mut ch = [0;1000];
            while let Ok(n) = read(self.pipe_infd, &mut ch) {
                ans += &String::from_utf8(ch[..n].to_vec()).unwrap();
                if n < 1000 {
                    break;
                };
            };
        }

        match waitpid(child, None)
            .expect("Faild to wait child process.") {
            WaitStatus::Exited(pid, status) => {
                if status != 0 {
                    eprintln!("Pid: {:?}, Exit with {:?}", pid, status);
                };
            }
            WaitStatus::Signaled(pid, signal, _) => {
                eprintln!("Pid: {:?}, Signal: {:?}", pid, signal)
            }
            _ => {
                eprintln!("Unknown error")
            }
        };

        ans
    }
}

