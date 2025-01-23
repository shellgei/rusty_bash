//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use crate::elements::word::Word;
use crate::utils::exit;
use nix::unistd;
use std::ffi::CString;
use std::process;

use nix::unistd::Pid;
use nix::errno::Errno;

fn reserved(w: &str) -> bool {
    matches!(w, "{" | "}" | "while" | "do" | "done" | "if" | "then" | "elif" | "else" | "fi")
}

#[derive(Debug, Default, Clone)]
pub struct SimpleCommand {
    text: String,
    words: Vec<Word>,
    args: Vec<String>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        self.args.clear();
        let mut words = self.words.to_vec();

        for w in words.iter_mut() {
            self.args.append(&mut w.eval(core).unwrap());
        }

        if self.args.is_empty() {
            return Ok(None);
        }

        if self.force_fork 
        || pipe.is_connected() 
        || ! core.builtins.contains_key(&self.args[0]) {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core)
        }
    }

    fn run(&mut self, core: &mut ShellCore, fork: bool) -> Result<(), ExecError> {
        if ! fork {
            core.run_builtin(&mut self.args);
            return Ok(());
        }

        if core.run_builtin(&mut self.args) {
            exit::normal(core)
        }else{
            Self::exec_external_command(&mut self.args)
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl SimpleCommand {
    fn exec_external_command(args: &mut [String]) -> ! {
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

    fn to_cargs(args: &mut [String]) -> Vec<CString> {
        args.iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect()
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        let w = match Word::parse(feeder, core)? {
            Some(w) => w,
            _       => return Ok(false),
        };

        if ans.words.is_empty() && reserved(&w.text) {
            return Ok(false);
        }
        ans.text += &w.text;
        ans.words.push(w);
        Ok(true)
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
        -> Result<Option<Self>, ParseError> {
        let mut ans = Self::default();
        feeder.set_backup();

        loop {
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            if ! Self::eat_word(feeder, &mut ans, core)? {
                break;
            }
        }

        if ans.words.len() + ans.redirects.len() > 0 {
            feeder.pop_backup();
            Ok(Some(ans))
        }else{
            feeder.rewind();
            Ok(None)
        }
    }
}
