//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod parser;
mod run_internal;

use crate::{ShellCore, proc_ctrl};
use crate::error::exec::ExecError;
use super::{Command, Pipe, Redirect};
use crate::elements::substitution::Substitution;
use crate::elements::word::Word;
use crate::utils::exit;
use nix::unistd::Pid;

#[derive(Debug, Clone)]
enum SubsArgType {
    Subs(Box<Substitution>), //clippyの指示でBoxに
    Other(Word),
}

#[derive(Debug, Default, Clone)]
pub struct SimpleCommand {
    text: String,
    substitutions: Vec<Substitution>,
    words: Vec<Word>,
    args: Vec<String>,
    redirects: Vec<Redirect>,
    force_fork: bool,
    substitutions_as_args: Vec<SubsArgType>,
    command_name: String,
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
                if let Err(e) = sub.clone().eval(core, None) {
                    let _ = core.db.set_param("?", "1", Some(0));
                    return Err(e);
                }
            }

            return Ok(None);
        }

        if self.force_fork 
        || pipe.is_connected() 
        || ( ! core.builtins.contains_key(&self.args[0]) 
             && ! core.subst_builtins.contains_key(&self.args[0])
             && ! core.db.functions.contains_key(&self.args[0]) ) {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core)
        }
    }

    fn run(&mut self, core: &mut ShellCore,
           fork: bool) -> Result<(), ExecError> {
        core.db.push_local();
        if let Err(e) = self.set_local_params(core) {
            e.print(core);
        }

        if !run_internal::run(self, core)? {
            self.set_environment_variables(core)?;
            proc_ctrl::exec_command(&self.args);
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
    fn set_local_params(&mut self,core: &mut ShellCore) -> Result<(), ExecError> {
        let layer = Some(core.db.get_layer_num() - 1);
        for s in self.substitutions.iter_mut() {
            s.eval(core, layer)?;
        }   
        Ok(())
    }

    fn set_environment_variables(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        let layer = core.db.get_layer_num() - 1;
        for entry in core.db.get_param_layer_ref(layer) {
            std::env::set_var(entry.0, entry.1.get_as_single()?);
        }
        Ok(())
    } 
}
