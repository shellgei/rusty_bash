//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod parser;

use crate::{proc_ctrl, ShellCore};

use crate::error::exec::ExecError;
use crate::utils;
use crate::utils::exit;
use super::{Command, Pipe, Redirect};
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;
use std::sync::atomic::Ordering::Relaxed;
use nix::unistd::Pid;

#[derive(Debug, Clone, Default)]
pub struct SimpleCommand {
    text: String,
    substitutions: Vec<Substitution>,
    words: Vec<Word>,
    pub args: Vec<String>,
    redirects: Vec<Redirect>,
    force_fork: bool, 
    substitutions_as_args: Vec<Substitution>,
    command_name: String,
    lineno: usize,
    continue_alias_check: bool,
    invalid_alias: bool,
    command_path: String,
}


impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe)
    -> Result<Option<Pid>, ExecError> {
        core.db.set_param("LINENO", &self.lineno.to_string(), None)?;
        if Self::break_continue_or_return(core) {
            return Ok(None);
        }

        core.db.set_param("BASH_COMMAND", &self.text, None)?;

        self.args.clear();
        let mut words = self.words.to_vec();
        for w in words.iter_mut() {
            self.set_arg(w, core)?;
        }

        if ! self.args.is_empty() && self.args[0].starts_with("%") {
            self.redirects.clear();
            self.args.insert(0, "fg".to_string());
        }

        match self.args.len() {
            0 => self.exec_set_param(core),
            _ => self.exec_command(core, pipe),
        }
    }

    fn run(&mut self, core: &mut ShellCore, fork: bool) -> Result<(), ExecError> {
        core.db.push_local();
        let layer = core.db.get_layer_num()-1;
        let _ = self.set_local_params(core, layer);

        if ! core.run_function(&mut self.args) 
        && ! core.run_builtin(&mut self.args, &self.substitutions_as_args)? {
            self.set_environment_variables(core)?;
            proc_ctrl::exec_command(&self.args, core, &self.command_path);
        };

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
    fn break_continue_or_return(core: &mut ShellCore) -> bool {
        core.break_counter > 0 || core.continue_counter > 0 
    }

    pub fn exec_command(&mut self, core: &mut ShellCore, pipe: &mut Pipe)
    -> Result<Option<Pid>, ExecError> {
        Self::check_sigint(core)?;

        core.db.last_arg = self.args.last().unwrap().clone();
        self.option_x_output(core);

        if core.db.flags.contains('r') {
            if self.args[0].contains('/') {
                let msg = format!("{}: restricted: cannot specify `/' in command names", &self.args[0]);
                return Err(ExecError::Other(msg));
            }
        }

        if self.force_fork 
        || pipe.is_connected() 
        || ( ! core.builtins.contains_key(&self.args[0]) 
           && ! core.db.functions.contains_key(&self.args[0]) ) {
            self.command_path = self.hash_control(core)?;
            self.fork_exec(core, pipe)
        }else if self.args.len() == 1 && self.args[0] == "exec" {
            for r in self.get_redirects().iter_mut() {
                r.connect(true, core)?;
            }
            Ok(None)
        }else{
            self.nofork_exec(core)
        }
    }

    fn check_sigint(core: &mut ShellCore) -> Result<(), ExecError> {
        if core.sigint.load(Relaxed) {
            core.db.exit_status = 130;
            return Err(ExecError::Interrupted);
        }
        Ok(())
    }

    fn exec_set_param(&mut self, core: &mut ShellCore) -> Result<Option<Pid>, ExecError> {
        core.db.last_arg = String::new();
        self.option_x_output(core);
        
        for s in self.substitutions.iter_mut() {
            if let Err(e) = s.eval(core, None, false) {
                e.print(core);
                core.db.exit_status = 1;
            }
        }

        Ok(None)
    }

    fn set_local_params(&mut self, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
        let mut layer = Some(layer);
        if core.options.query("posix") {
            layer = None;
        }
        for s in self.substitutions.iter_mut() {
            s.eval(core, layer, false)?;
        }
        Ok(())
    }

    fn set_environment_variables(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        for s in self.substitutions.iter_mut() {
            s.eval(core, None, true)?;
        }
        Ok(())
    }

    fn set_arg(&mut self, word: &mut Word, core: &mut ShellCore) -> Result<(), ExecError> {
        match word.eval(core) {
            Ok(ws) => {
                self.args.extend(ws);
                Ok(())
            },
            Err(e) => {
                e.print(core);
                if ! core.sigint.load(Relaxed) {
                    core.db.exit_status = 1;
                }
                Err(e)
            },
        }
    }

    fn option_x_output(&self, core: &mut ShellCore) {
        if ! core.db.flags.contains('x') {
            return;
        }

        let ps4 = core.get_ps4();
        for s in &self.substitutions {
            eprintln!("\r{} {}\r", &ps4, &s.text);
        }

        if self.args.is_empty() {
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

    fn hash_control(&mut self, core: &mut ShellCore) -> Result<String, ExecError> {
        if self.args[0].starts_with("/")
        || self.args[0].starts_with("./")
        || self.args[0].starts_with("../") {
            return Ok(self.args[0].clone());
        }

        let hash = core.db.get_array_elem("BASH_CMDS", &self.args[0])?;

        let restricted = core.db.flags.contains('r');
        core.db.flags.retain(|f| f != 'r');
        if hash.is_empty() {
            let path = utils::get_command_path(&self.args[0], core);
            if path != "" {
                core.db.set_assoc_elem("BASH_CMDS", &self.args[0], &path, None)?;
                core.db.hash_counter.insert(self.args[0].clone(), 1);
                if restricted {
                    core.db.flags.push('r');
                }
                return Ok(path);
            }
        }else{
            match core.db.hash_counter.get_mut(&self.args[0]) {
                Some(v) => *v += 1,
                None => {core.db.hash_counter.insert(self.args[0].clone(), 1);},
            }
            if restricted {
                core.db.flags.push('r');
            }
            return Ok(hash);
        }
        if restricted {
            core.db.flags.push('r');
        }
        Ok(String::new())
    }
}
