//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd;
use std::ffi::CString;
use std::process;

use nix::unistd::{ForkResult, Pid};
use nix::errno::Errno;

#[derive(Debug)]
pub struct SimpleCommand {
    pub text: String,
    args: Vec<String>,
    cargs: Vec<CString>,
    redirects: Vec<Redirect>,
}

impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.args.len() == 0 {
            return None;
        }
        if ! pipe.is_connected() && core.builtins.contains_key(&self.args[0]){
            if self.redirects.iter_mut().all(|r| r.connect(true)){
                core.run_builtin(&mut self.args);
            }else{
                core.vars.insert("?".to_string(), "1".to_string());
            }
            self.redirects.iter_mut().rev().for_each(|r| r.restore());
            return None;
        }

        self.set_cargs();
        match unsafe{unistd::fork()} {
            Ok(ForkResult::Child) => {
                if let Err(_) = unistd::setpgid(Pid::from_raw(0), pipe.pgid) {
                    panic!("sush(fatal): cannot set pgid");
                }

                if ! self.redirects.iter_mut().all(|r| r.connect(false)){
                    process::exit(1);
                }
                pipe.connect();
                if core.run_builtin(&mut self.args) {
                    core.exit();
                }

                match unistd::execvp(&self.cargs[0], &self.cargs) {
                    Err(Errno::EACCES) => {
                        println!("sush: {}: Permission denied", &self.args[0]);
                        process::exit(126)
                    },
                    Err(Errno::ENOENT) => {
                        println!("{}: command not found", &self.args[0]);
                        process::exit(127)
                    },
                    Err(err) => {
                        println!("Failed to execute. {:?}", err);
                        process::exit(127)
                    }
                    _ => panic!("SUSH INTERNAL ERROR (never come here)") // _ => () 
                }
            },
            Ok(ForkResult::Parent { child } ) => {
                if let Err(_) = unistd::setpgid(child, pipe.pgid) {
                    panic!("sush(fatal): cannot set pgid");
                }

                pipe.parent_close();
                Some(child) //core.wait_process(child);
            },
            Err(err) => panic!("sush(fatal): Failed to fork. {}", err),
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl SimpleCommand {
    fn set_cargs(&mut self) {
        self.cargs = self.args.iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect();
    }

    fn new() -> SimpleCommand {
        SimpleCommand {
            text: String::new(),
            args: vec![],
            cargs: vec![],
            redirects: vec![],
        }
    }
 
    fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand, core: &mut ShellCore) -> bool {
        let arg_len = feeder.scanner_word(core);
        if arg_len == 0 {
            return false;
        }
 
        let word = feeder.consume(arg_len);
        if ans.args.len() == 0 && ( word == "{" || word == "}") {
            return false;
        }
 
        ans.text += &word.clone();
        ans.args.push(word);
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SimpleCommand> {
        let mut ans = Self::new();
        let backup = feeder.clone();

        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if ! command::eat_redirect(feeder, core, &mut ans.redirects, &mut ans.text)
                && ! Self::eat_word(feeder, &mut ans, core) {
                break;
            }
        }

        if ans.args.len() + ans.redirects.len() > 0 {
//            eprintln!("{:?}", ans);
            Some(ans)
        }else{
            feeder.rewind(backup);
            None
        }
    }
}
