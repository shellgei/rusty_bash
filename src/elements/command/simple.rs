//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use nix::unistd;
use std::ffi::CString;
use std::process;

use nix::unistd::Pid;
use nix::errno::Errno;

fn reserved(w: &str) -> bool {
    match w {
        "{" | "}" | "while" | "do" | "done" | "if" | "then" | "elif" | "else" | "fi" => true,
        _ => false,
    }
}

#[derive(Debug)]
pub struct SimpleCommand {
    text: String,
    args: Vec<String>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.args.len() == 0 {
            return None;
        }

        if self.force_fork 
        || pipe.is_connected() 
        || ! core.builtins.contains_key(&self.args[0]) {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core);
            None
        }
    }

    fn run_command(&mut self, core: &mut ShellCore, fork: bool) {
        if ! fork {
            core.run_builtin(&mut self.args);
            return;
        }

        if core.run_builtin(&mut self.args) {
            core.exit()
        }else{
            Self::exec_external_command(&mut self.args)
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
}

impl SimpleCommand {
    fn exec_external_command(args: &mut Vec<String>) -> ! {
        let cargs = Self::to_cargs(args);
        match unistd::execvp(&cargs[0], &cargs) {
            Err(Errno::EACCES) => {
                println!("sush: {}: Permission denied", &args[0]);
                process::exit(126)
            },
            Err(Errno::ENOENT) => {
                println!("{}: command not found", &args[0]);
                process::exit(127)
            },
            Err(err) => {
                println!("Failed to execute. {:?}", err);
                process::exit(127)
            }
            _ => panic!("SUSH INTERNAL ERROR (never come here)")
        }
    }

    fn to_cargs(args: &mut Vec<String>) -> Vec<CString> {
        args.iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect()
    }

    fn new() -> SimpleCommand {
        SimpleCommand {
            text: String::new(),
            args: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }
 
    fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand, core: &mut ShellCore) -> bool {
        let arg_len = feeder.scanner_word(core);
        if arg_len == 0 {
            return false;
        }
 
        let word = feeder.consume(arg_len);
        if ans.args.len() == 0 && reserved(&word) {
            return false;
        }
 
        ans.text += &word.clone();
        ans.args.push(word);
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SimpleCommand> {
        let mut ans = Self::new();
        feeder.set_backup();

        loop {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
            if ! command::eat_redirect(feeder, core, &mut ans.redirects, &mut ans.text)
                && ! Self::eat_word(feeder, &mut ans, core) {
                break;
            }
        }

        if ans.args.len() + ans.redirects.len() > 0 {
            feeder.pop_backup();
            Some(ans)
        }else{
            feeder.rewind();
            None
        }
    }
}
