//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod parser;

use crate::{error, proc_ctrl, ShellCore};
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
    permit_substitution_arg: bool,
    lineno: usize,
}


impl Command for SimpleCommand {
    fn exec(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        let _ = core.db.set_param("LINENO", &self.lineno.to_string(), None);
        if Self::break_continue_or_return(core) {
            return None;
        }

        self.args.clear();
        let mut words = self.words.to_vec();
        if ! words.iter_mut().all(|w| self.set_arg(w, core)){
            core.word_eval_error = true;
            return None;
        }

        match self.args.len() {
            0 => self.exec_set_param(core),
            _ => self.exec_command(core, pipe),
        }
    }

    fn run(&mut self, core: &mut ShellCore, fork: bool) {
        core.db.push_local();
        let layer = core.db.get_layer_num()-1;
        self.set_local_params(core, layer);

        if core.db.functions.contains_key(&self.args[0]) {
            let mut f = core.db.functions[&self.args[0]].clone();
            f.run_as_command(&mut self.args, core);
        } else if core.builtins.contains_key(&self.args[0]) {
            let mut special_args = self.substitutions_as_args.iter()
                                       .map(|a| a.text.clone()).collect();
            core.run_builtin(&mut self.args, &mut special_args);
        } else {
            self.set_environment_variables(core);
            proc_ctrl::exec_command(&self.args, core);
        }

        core.db.pop_local();

        if fork {
            exit::normal(core);
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
        core.return_flag || core.break_counter > 0 || core.continue_counter > 0 
    }

    pub fn exec_command(&mut self, core: &mut ShellCore, pipe: &mut Pipe) -> Option<Pid> {
        if Self::check_sigint(core) {
            return None;
        }

        core.db.last_arg = self.args.last().unwrap().clone();
        self.option_x_output(core);

        if self.force_fork 
        || pipe.is_connected() 
        || ( ! core.builtins.contains_key(&self.args[0]) 
           && ! core.db.functions.contains_key(&self.args[0]) ) {
            self.fork_exec(core, pipe)
        }else{
            self.nofork_exec(core);
            None
        }
    }

    fn check_sigint(core: &mut ShellCore) -> bool {
        if core.sigint.load(Relaxed) {
            core.db.exit_status = 130;
            return true;
        }
        false
    }

    fn exec_set_param(&mut self, core: &mut ShellCore) -> Option<Pid> {
        core.db.last_arg = String::new();
        self.option_x_output(core);
        
        self.substitutions.iter_mut()
            .for_each(|s| {s.eval(core, None, false);});

        None
    }

    fn set_local_params(&mut self, core: &mut ShellCore, layer: usize) {
        self.substitutions.iter_mut()
            .for_each(|s| {s.eval(core, Some(layer), false);});
    }

    fn set_environment_variables(&mut self, core: &mut ShellCore) {
        self.substitutions.iter_mut()
            .for_each(|s| {s.eval(core, None, true);} );
    }

    fn set_arg(&mut self, word: &mut Word, core: &mut ShellCore) -> bool {
        match word.eval(core) {
            Ok(ws) => {
                self.args.extend(ws);
                true
            },
            Err(e) => {
                error::print(&e, core);
                if ! core.sigint.load(Relaxed) {
                    core.db.exit_status = 1;
                }
                false
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
}
