//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod alias;
mod cd;
pub mod completion;
mod history;
mod job_commands;
pub mod parameter;
pub mod option;
mod pwd;
mod read;
mod source;
mod loop_control;
mod unset;
mod utils;

use crate::{Feeder, Script, ShellCore};
use crate::utils::{error, exit};

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("alias".to_string(), alias::alias);
        self.builtins.insert("bg".to_string(), job_commands::bg);
        self.builtins.insert("break".to_string(), loop_control::break_);
        self.builtins.insert("builtin".to_string(), builtin);
        self.builtins.insert("cd".to_string(), cd::cd);
        self.builtins.insert("compgen".to_string(), completion::compgen);
        self.builtins.insert("complete".to_string(), completion::complete);
        self.builtins.insert("continue".to_string(), loop_control::continue_);
        self.builtins.insert("declare".to_string(), parameter::declare);
        self.builtins.insert("eval".to_string(), eval);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("fg".to_string(), job_commands::fg);
        self.builtins.insert("history".to_string(), history::history);
        self.builtins.insert("jobs".to_string(), job_commands::jobs);
        self.builtins.insert("local".to_string(), parameter::local);
        self.builtins.insert("pwd".to_string(), pwd::pwd);
        self.builtins.insert("read".to_string(), read::read);
        self.builtins.insert("return".to_string(), loop_control::return_);
        self.builtins.insert("set".to_string(), option::set);
        self.builtins.insert("shift".to_string(), option::shift);
        self.builtins.insert("shopt".to_string(), option::shopt);
        self.builtins.insert("unalias".to_string(), alias::unalias);
        self.builtins.insert("unset".to_string(), unset::unset);
        self.builtins.insert("source".to_string(), source::source);
        self.builtins.insert(".".to_string(), source::source);
        self.builtins.insert("true".to_string(), true_);
        self.builtins.insert("wait".to_string(), job_commands::wait);
    }
}

pub fn builtin(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return 0;
    }

    if ! core.builtins.contains_key(&args[1]) {
        let msg = format!("{}: {}: not a shell builtin", &args[0], &args[1]);
        error::print(&msg, core);
        return 1;
    }

    core.builtins[&args[1]](core, &mut args[1..].to_vec())
}

pub fn eval(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut feeder = Feeder::new(&args[1..].join(" "));

    core.eval_level += 1;
    match Script::parse(&mut feeder, core, false){
        Some(mut s) => s.exec(core),
        None        => {},
    }

    core.eval_level -= 1;
    match core.db.get_param("?").parse::<i32>() {
        Ok(es) => es,
        _      => 1,
    }
}

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
    if args.len() > 1 {
        core.db.set_layer_param("?", &args[1], 0);
    }
    exit::normal(core)
}

pub fn false_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    1
}

pub fn true_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    0
}
