//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{Command, Pipe, Redirect};
use crate::elements::command;
use crate::elements::substitution::{Substitution, Value};
use crate::elements::word::Word;
use nix::unistd;
use std::collections::HashMap;
use std::ffi::CString;
use std::{env, process};
use std::sync::atomic::Ordering::Relaxed;

use nix::unistd::Pid;
use nix::errno::Errno;

fn target(w: &str) -> bool {
    match w {
        "local" => true,
        _ => false,
    }
}

#[derive(Debug, Clone)]
pub struct SubstituteCommand {
    text: String,
    substitutions: Vec<Substitution>,
    evaluated_subs: Vec<(String, Value)>,
    words: Vec<Word>,
    args: Vec<String>,
    redirects: Vec<Redirect>,
    force_fork: bool,
}

impl Command for SubstituteCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        self.args.clear();
        let mut words = self.words.to_vec();

        if ! self.eval_substitutions(core){
            core.data.set_param("?", "1");
            return None;
        }

        if ! words.iter_mut().all(|w| self.set_arg(w, core)){
            return None;
        }

        if self.args.len() == 0 {
            for s in &self.evaluated_subs {
                match &s.1 {
                    Value::EvaluatedSingle(v) => core.data.set_param(&s.0, &v),
                    Value::EvaluatedArray(a) => core.data.set_array(&s.0, &a),
                    _ => {},
                }
            }
            return None;
        }else if Self::check_sigint(core) {
            None
        }else if core.data.functions.contains_key(&self.args[0]) {
            self.exec_function(core, pipe)
        }else{
            self.exec_command(core, pipe)
        }
    }

    fn run(&mut self, core: &mut ShellCore, fork: bool) {
        if ! fork {
            core.data.parameters.push(HashMap::new());
            core.data.arrays.push(HashMap::new());

            for s in &self.evaluated_subs {
                match &s.1 {
                    Value::EvaluatedSingle(v) => core.data.set_local_param(&s.0, &v),
                    Value::EvaluatedArray(a) => core.data.set_local_array(&s.0, &a),
                    _ => {},
                }
            }

            core.run_builtin(&mut self.args);

            core.data.parameters.pop();
            core.data.arrays.pop();

            return;
        }

        match core.run_builtin(&mut self.args) {
            true  => core.exit(),
            false => self.exec_external_command(),
        }
    }

    fn get_text(&self) -> String { self.text.clone() }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> { &mut self.redirects }
    fn set_force_fork(&mut self) { self.force_fork = true; }
    fn boxed_clone(&self) -> Box<dyn Command> {Box::new(self.clone())}
}

impl SubstituteCommand {
    fn exec_external_command(&self) -> ! {
        let cargs = Self::to_cargs(&self.args);
        for s in &self.evaluated_subs {
            match &s.1 {
                Value::EvaluatedSingle(v) => env::set_var(&s.0, &v),
                _ => {},
            }
        }
        match unistd::execvp(&cargs[0], &cargs) {
            Err(Errno::E2BIG) => {
                println!("sush: {}: Arg list too long", &self.args[0]);
                process::exit(126)
            },
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
            _ => panic!("SUSH INTERNAL ERROR (never come here)")
        }
    }

    fn exec_command(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if self.force_fork 
        || pipe.is_connected() 
        || ! core.builtins.contains_key(&self.args[0]) {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core);
            None
        }
    }

    fn exec_function(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        let mut command = core.data.functions[&self.args[0]].clone();
        command.run_as_command(&mut self.args, core, Some(pipe))
    }

    fn check_sigint(core: &mut ShellCore) -> bool {
        if core.sigint.load(Relaxed) {
            core.data.set_param("?", "130");
            return true;
        }
        false
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

    fn new() -> SubstituteCommand {
        SubstituteCommand {
            text: String::new(),
            substitutions: vec![],
            evaluated_subs: vec![],
            words: vec![],
            args: vec![],
            redirects: vec![],
            force_fork: false,
        }
    }

    fn eat_substitution(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(s) = Substitution::parse(feeder, core) {
            ans.text += &s.text;
            ans.substitutions.push(s);
            true
        }else{
            false
        }
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut SubstituteCommand, core: &mut ShellCore) -> bool {
        let w = match Word::parse(feeder, core) {
            Some(w) => w,
            _       => return false,
        };

        if ans.words.len() == 0 && ! target(&w.text) {
            return false;
        }
        ans.text += &w.text;
        ans.words.push(w);
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SubstituteCommand> {
        let mut ans = Self::new();
        feeder.set_backup();

        while Self::eat_substitution(feeder, &mut ans, core) {
            command::eat_blank_with_comment(feeder, core, &mut ans.text);
        }

        loop {
            command::eat_redirects(feeder, core, &mut ans.redirects, &mut ans.text);
            if ! Self::eat_word(feeder, &mut ans, core) {
                break;
            }
        }

        if ans.substitutions.len() + ans.words.len() + ans.redirects.len() > 0 {
            feeder.pop_backup();
            Some(ans)
        }else{
            feeder.rewind();
            None
        }
    }
}
