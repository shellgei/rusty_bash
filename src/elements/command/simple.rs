//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod alias;
pub mod hash;
pub mod parser;
pub mod run_internal;

use crate::{proc_ctrl, ShellCore};

use super::{Command, Pipe, Redirect};
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;
use crate::error::exec::ExecError;
use crate::utils::exit;
use nix::unistd::Pid;
use std::sync::atomic::Ordering::Relaxed;

#[derive(Debug, Clone)]
enum SubsArgType {
    Subs(Box<Substitution>),
    Other(Word),
}

#[derive(Debug, Clone, Default)]
pub struct SimpleCommand {
    text: String,
    substitutions: Vec<Substitution>,
    words: Vec<Word>,
    pub args: Vec<String>,
    redirects: Vec<Redirect>,
    force_fork: bool,
    substitutions_as_args: Vec<SubsArgType>,
    command_name: String,
    pub lineno: usize,
    continue_alias_check: bool,
    invalid_alias: bool,
    command_path: String,
}

impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Result<Option<Pid>, ExecError> {
        core.db
            .set_param("LINENO", &self.lineno.to_string(), None)?;
        if Self::break_continue_or_return(core) {
            return Ok(None);
        }

        core.db.set_param("BASH_COMMAND", &self.text, None)?;

        self.args.clear();
        let mut words = self.words.to_vec();
        for w in words.iter_mut() {
            w.set_pipe(core); //for >()
            self.set_arg(w, core)?;
        }

        if !self.args.is_empty() && self.args[0].starts_with("%") {
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
        let _ = self.set_local_params(core);

        if !run_internal::run(self, core)? {
            self.set_environment_variables(core)?;
            proc_ctrl::exec_command(&self.args, core, &self.command_path);
        };

        core.db.pop_local();

        match fork {
            true => exit::normal(core),
            false => Ok(()),
        }
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
    fn get_redirects(&mut self) -> &mut Vec<Redirect> {
        &mut self.redirects
    }
    fn set_force_fork(&mut self) {
        self.force_fork = true;
    }
    fn boxed_clone(&self) -> Box<dyn Command> {
        Box::new(self.clone())
    }
    fn force_fork(&self) -> bool {
        self.force_fork
    }

    fn get_lineno(&mut self) -> usize {
        self.lineno
    }
}

impl SimpleCommand {
    fn break_continue_or_return(core: &mut ShellCore) -> bool {
        core.break_counter > 0 || core.continue_counter > 0
    }

    pub fn exec_command(
        &mut self,
        core: &mut ShellCore,
        pipe: &mut Pipe,
    ) -> Result<Option<Pid>, ExecError> {
        Self::check_sigint(core)?;

        core.db.last_arg = self.args.last().unwrap().clone();
        self.option_x_output(core);

        if core.db.flags.contains('r') && self.args[0].contains('/') {
            let msg = format!(
                "{}: restricted: cannot specify `/' in command names",
                &self.args[0]
            );
            return Err(ExecError::Other(msg));
        }

        if self.args[0] == "command" && self.args.len() > 1 {
            if core.subst_builtins.contains_key(&self.args[1])
            || core.db.functions.contains_key(&self.args[1]) {
                self.args.remove(0);
            }
        }

        if self.force_fork
            || (!pipe.lastpipe && pipe.is_connected())
            || (!core.builtins.contains_key(&self.args[0])
                && !core.subst_builtins.contains_key(&self.args[0])
                && !core.db.functions.contains_key(&self.args[0]))
        {
            self.command_path = hash::get_and_regist(self, core)?;
            self.fork_exec(core, pipe)
        } else if self.args.len() == 1 && self.args[0] == "exec" {
            for r in self.get_redirects().iter_mut() {
                if let Err(e) = r.connect(true, core) {
                    e.print(core);
                    core.db.exit_status = 1;
                    break;
                }
            }
            Ok(None)
        } else {
            if let Err(e) = pipe.connect_lastpipe(core) {
                e.print(core);
                core.db.exit_status = 1;
            }
            if let Err(e) = self.nofork_exec(core) {
                e.print(core);
                core.db.exit_status = 1;
            }
            Ok(None)
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
                core.db.exit_status = 1;
                if !core.db.flags.contains('i') {
                    if let ExecError::SyntaxError(_) = e {
                        e.print(core);
                        let msg = "`".to_owned() + &s.text.clone() + "'";
                        return Err(ExecError::Other(msg));
                    }
                }
                return Err(e);
            }
        }

        Ok(None)
    }

    fn set_local_params(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let layer = core.db.get_layer_num() - 1;
        if core.options.query("posix") {
            for s in self.substitutions.clone().iter_mut() {
                s.eval(core, None, false)?;
            }
        }
        for s in self.substitutions.iter_mut() {
            s.eval(core, Some(layer), false)?;
        }

        Ok(())
    }

    fn set_environment_variables(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let layer = core.db.get_layer_num() - 1;
        core.db.set_layer_to_env(layer);
        Ok(())
    }

    fn set_arg(&mut self, word: &mut Word, core: &mut ShellCore) -> Result<(), ExecError> {
        match word.eval(core) {
            Ok(ws) => {
                self.args.extend(ws);
                Ok(())
            }
            Err(e) => {
                //   e.print(core);
                if !core.sigint.load(Relaxed) {
                    core.db.exit_status = 1;
                }
                Err(e)
            }
        }
    }

    fn option_x_output(&self, core: &mut ShellCore) {
        if !core.db.flags.contains('x') {
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
            match a.contains(" ") {
                false => eprint!(" {}", &a),
                true => eprint!(" '{}'", &a),
            }
        }

        eprintln!();
    }
}
