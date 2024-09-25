//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod parser;

use crate::{error, ShellCore};
use super::{Command, Pipe, Redirect};
use crate::core::data::Value;
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;
use nix::unistd;
use std::ffi::CString;
use std::{env, process};
use std::sync::atomic::Ordering::Relaxed;

use nix::unistd::Pid;
use nix::errno::Errno;

#[derive(Debug, Clone)]
pub struct SimpleCommand {
    text: String,
    substitutions: Vec<Substitution>,
    evaluated_subs: Vec<(String, Value)>,
    words: Vec<Word>,
    args: Vec<String>,
    redirects: Vec<Redirect>,
    force_fork: bool, 
    substitutions_as_args: Vec<Substitution>,
    permit_substitution_arg: bool,
}


impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if core.return_flag || core.break_counter > 0 {
            return None;
        }

        if ! self.eval_substitutions(core){
            core.data.set_param("?", "1");
            return None;
        }

        self.args.clear();
        let mut words = self.words.to_vec();
        if ! words.iter_mut().all(|w| self.set_arg(w, core)){
            core.word_eval_error = true;
            return None;
        }

        if self.args.len() == 0 {
            core.data.set_param("_", "");
            self.option_x_output(core);
            self.exec_set_params(core)
        }else if Self::check_sigint(core) {
            None
        }else{
            core.data.set_param("_", &self.args.last().unwrap());
            self.option_x_output(core);
            self.exec_command(core, pipe)
        }
    }

    fn run(&mut self, core: &mut ShellCore, fork: bool) {
        core.data.push_local();
        self.set_local_params(core);

        if core.data.functions.contains_key(&self.args[0]) {
            let mut f = core.data.functions[&self.args[0]].clone();
            f.run_as_command(&mut self.args, core);
        } else if core.builtins.contains_key(&self.args[0]) {
            let mut special_args = self.substitutions_as_args.iter().map(|a| a.text.clone()).collect();
            core.run_builtin(&mut self.args, &mut special_args);
        } else {
            self.exec_external_command(core);
        }

        core.data.pop_local();

        if fork {
            core.exit();
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
    fn force_fork(&self) -> bool { self.force_fork }
}

impl SimpleCommand {
    fn exec_external_command(&mut self, core: &mut ShellCore) -> ! {
        self.set_environment_variables();
        let cargs = Self::to_cargs(&self.args);

        match unistd::execvp(&cargs[0], &cargs) {
            Err(Errno::E2BIG) => error::arg_list_too_long(&self.args[0], core),
            Err(Errno::EACCES) => {
                eprintln!("sush: {}: Permission denied", &self.args[0]);
                process::exit(126)
            },
            Err(Errno::ENOENT) => {
                let msg = format!("{}: command not found", &self.args[0]);
                error::print(&msg, core, false);
                process::exit(127)
            },
            Err(err) => {
                eprintln!("Failed to execute. {:?}", err);
                process::exit(127)
            }
            _ => error::internal("never come here")
        }
    }

    fn exec_command(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.force_fork 
        || pipe.is_connected() 
        || ( ! core.builtins.contains_key(&self.args[0]) 
           && ! core.data.functions.contains_key(&self.args[0]) ) {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core);
            None
        }
    }

    fn check_sigint(core: &mut ShellCore) -> bool {
        if core.sigint.load(Relaxed) {
            core.data.set_param("?", "130");
            return true;
        }
        false
    }

    fn exec_set_params(&mut self, core: &mut ShellCore) -> Option<Pid> {
        for s in &self.evaluated_subs {
            match &s.1 {
                Value::EvaluatedSingle(v) => core.data.set_param(&s.0, &v),
                Value::EvaluatedArray(a) => core.data.set_array(&s.0, &a),
                _ => {},
            }
        }
        None
    }

    fn set_local_params(&mut self, core: &mut ShellCore) {
        for s in &self.evaluated_subs {
            match &s.1 {
                Value::EvaluatedSingle(v) => core.data.set_local_param(&s.0, &v),
                Value::EvaluatedArray(a) => core.data.set_local_array(&s.0, &a),
                _ => {},
            }
        }
    }

    fn set_environment_variables(&mut self) {
        for s in &self.evaluated_subs {
            match &s.1 {
                Value::EvaluatedSingle(v) => env::set_var(&s.0, &v),
                _ => {},
            }
        }
    }

    fn to_cargs(args: &Vec<String>) -> Vec<CString> {
        args.iter()
            .map(|a| CString::new(a.to_string()).unwrap())
            .collect()
    }

    fn eval_substitutions(&mut self, core: &mut ShellCore) -> bool {
        self.evaluated_subs.clear();
        for s in &mut self.substitutions {
            match s.eval(core) {
                Value::None => return false,
                a           => self.evaluated_subs.push( (s.key.clone(), a) ),
            }
        }
        true
    }

    fn set_arg(&mut self, word: &mut Word, core: &mut ShellCore) -> bool {
        match word.eval(core) {
            Some(ws) => {
                self.args.extend(ws);
                true
            },
            None => {
                if ! core.sigint.load(Relaxed) {
                    core.data.set_param("?", "1");
                }
                false
            },
        }
    }

    fn option_x_output(&self, core: &mut ShellCore) {
        if ! core.data.flags.contains('x') {
            return;
        }

        let ps4 = core.get_ps4();
        for s in &self.substitutions {
            eprintln!("{} {}", &ps4, &s.text);
        }

        if self.args.len() == 0 {
            return;
        }

        eprint!("{}", &ps4);
        for a in &self.args {
            match a.contains(" "){
                false => eprint!(" {}", &a),
                true  => eprint!(" '{}'", &a),
            }
        }
        eprintln!("");
    }
}
