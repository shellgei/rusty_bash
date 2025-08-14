//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;
use crate::utils;
use crate::utils::exit;
use nix::unistd;
use std::ffi::CString;
use std::process;

use nix::unistd::Pid;
use nix::errno::Errno;

#[derive(Debug, Default, Clone)]
pub struct SimpleCommand {
    text: String,
    substitutions: Vec<Substitution>,
    words: Vec<Word>,
    args: Vec<String>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe)
    -> Result<Option<Pid>, ExecError> {
        self.args.clear();
        let mut words = self.words.to_vec();
        for w in words.iter_mut() {
            let mut args = w.eval(core)?;
            self.args.append(&mut args);
        }

        if self.args.is_empty() {
            for sub in &self.substitutions {
                sub.clone().eval(core)?;
            }

            return Ok(None);
        }

        if self.force_fork 
        || pipe.is_connected() 
        || ( ! core.builtins.contains_key(&self.args[0]) 
             && ! core.db.functions.contains_key(&self.args[0]) ) {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core)
        }
    }

    fn run(&mut self, core: &mut ShellCore, fork: bool) -> Result<(), ExecError> {
        core.db.push_local();
        let layer = core.db.get_layer_num() - 1;
        let _ = self.set_local_params(core, layer);

        if ! core.run_function(&mut self.args) 
        && ! core.run_builtin(&mut self.args) {
            Self::exec_external_command(&mut self.args)
        }

        core.db.pop_local();

        match fork {
            true  => exit::normal(core),
            false => Ok(()),
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

    fn set_local_params(&mut self, core: &mut ShellCore,
                        layer: usize) -> Result<(), ExecError> {
        let mut layer = Some(layer);
        for s in self.substitutions.iter_mut() {
            s.eval(core, layer, false)?;
        }   
        Ok(())
    }

    fn to_cargs(args: &mut [String]) -> Vec<CString> {
        args.iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect()
    }

    pub fn eat_substitution(&mut self, feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<bool, ParseError> {
        if let Some(s) = Substitution::parse(feeder, core)? {
            self.text += &s.text;
            self.substitutions.push(s);
            Ok(true)
        }else{
            Ok(false)
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut SimpleCommand, core: &mut ShellCore)
        -> Result<bool, ParseError> {
        let w = match Word::parse(feeder, core)? {
            Some(w) => w,
            _       => return Ok(false),
        };

        if ans.words.is_empty() && utils::reserved(&w.text) {
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
            if ! ans.eat_substitution(feeder, core)? {
                break;
            }
        }

        loop {
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text)?;
            if ! Self::eat_word(feeder, &mut ans, core)? {
                break;
            }
        }

        if ans.words.len() + ans.redirects.len() + ans.substitutions.len() > 0 {
            feeder.pop_backup();
            Ok(Some(ans))
        }else{
            feeder.rewind();
            Ok(None)
        }
    }
}
