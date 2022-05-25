//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use nix::unistd::{execvpe, fork, ForkResult, Pid}; 
use nix::sys::wait::*;
use std::ffi::CString;
use std::process::exit;
use std::env;

use crate::{ShellCore,Feeder,CommandPart};
use crate::utils::blue_string;
use crate::elems_in_command::{Arg, Substitution};

pub trait Executable {
    fn eval(&self, _conf: &mut ShellCore) -> Vec<String> { vec!() }
    fn exec(&self, _conf: &mut ShellCore) {}
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
    fn exec(&self, conf: &mut ShellCore) {
        if conf.flags.d {
            eprintln!("{}", self.parse_info().join("\n"));
        };

        for e in &self.elems {
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
    pub elems: Vec<Box<dyn CommandPart>>,
    text: String,
    //pub debug: DebugInfo,
}

impl Executable for CommandWithArgs {
    fn eval(&self, conf: &mut ShellCore) -> Vec<String> {
        let mut args = vec!();

        for elem in &self.elems {
            for s in &elem.eval(conf) {
                args.append(&mut Arg::expand_glob(&s.clone()));
            }
        };

        args.iter()
            .map(|a| Arg::remove_escape(&a))
            .collect()
    }

    fn exec(&self, conf: &mut ShellCore){
        let mut args = self.eval(conf);

        if let Some(func) = conf.get_internal_command(&args[0]) {
            func(&mut args);
            return;
        }

        unsafe {
            match fork() {
                Ok(ForkResult::Child) => self.exec_external_command(&args, &self.vars, conf),
                Ok(ForkResult::Parent { child } ) => CommandWithArgs::wait_command(child),
                Err(err) => panic!("Failed to fork. {}", err),
            }
        }
    }
}

impl CommandWithArgs {
    pub fn new() -> CommandWithArgs{
        CommandWithArgs {
            vars: vec!(),
            elems: vec!(),
            text: "".to_string(),
        }
    }

    pub fn push_vars(&mut self, s: Substitution){
        self.text += &s.text();
        self.vars.push(Box::new(s));
    }

    pub fn push_elems(&mut self, s: Box<dyn CommandPart>){
        self.text += &s.text();
        self.elems.push(s);
    }

    pub fn return_if_valid(ans: CommandWithArgs, text: &mut Feeder, backup: Feeder) -> Option<CommandWithArgs> {
        if ans.elems.len() > 0 {
              Some(ans)
        }else{
            text.rewind(backup);
            None
        }
    }

    fn parse_info(&self) -> Vec<String> {
        let mut ans = vec!(format!("command: '{}'", self.text));
        for elem in &self.elems {
            ans.append(&mut elem.parse_info());
        };
        
        blue_string(&ans)
    }

    fn exec_external_command(&self, args: &Vec<String>, vars: &Vec<Box<Substitution>>, conf: &mut ShellCore) {
        let cargs: Vec<CString> = args
            .iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();

        if conf.flags.d {
            eprintln!("{}", self.parse_info().join("\n"));
        };

        for v in vars {
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

    fn wait_command(child: Pid) {
        match waitpid(child, None)
            .expect("Faild to wait child process.") {
            WaitStatus::Exited(pid, status) => {
                if status != 0 {
                    println!("Pid: {:?}, Exit with {:?}", pid, status);
                };
            }
            WaitStatus::Signaled(pid, signal, _) => {
                println!("Pid: {:?}, Signal: {:?}", pid, signal)
            }
            _ => {
                println!("Unknown error")
            }
        };
    }
}

