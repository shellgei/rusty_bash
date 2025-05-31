//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod alias;
mod cd;
pub mod compgen;
pub mod complete;
mod compopt;
mod command;
mod exec;
mod getopts;
mod hash;
mod history;
mod job_commands;
pub mod parameter;
pub mod option;
mod printf;
mod pwd;
mod read;
mod source;
mod trap;
mod type_;
mod loop_control;
mod unset;

use crate::{exit, Feeder, Script, ShellCore};
use crate::elements::expr::arithmetic::ArithmeticExpr;

pub fn error_exit(exit_status: i32, name: &str, msg: &str, core: &mut ShellCore) -> i32 {
    let shellname = core.db.get_param("0").unwrap();
    if core.db.flags.contains('i') {
        eprintln!("{}: {}: {}", &shellname, name, msg);
    }else{
        let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
        eprintln!("{}: line {}: {}: {}", &shellname, &lineno, name, msg);
    }
    exit_status
}

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("alias".to_string(), alias::alias);
        self.builtins.insert("bg".to_string(), job_commands::bg);
        self.builtins.insert("bind".to_string(), bind);
        self.builtins.insert("break".to_string(), loop_control::break_);
        self.builtins.insert("builtin".to_string(), command::builtin);
        self.builtins.insert("cd".to_string(), cd::cd);
        self.builtins.insert("command".to_string(), command::command);
        self.builtins.insert("compgen".to_string(), compgen::compgen);
        self.builtins.insert("complete".to_string(), complete::complete);
        self.builtins.insert("compopt".to_string(), compopt::compopt);
        self.builtins.insert("continue".to_string(), loop_control::continue_);
        self.builtins.insert("debug".to_string(), debug);
        self.builtins.insert("disown".to_string(), job_commands::disown);
        self.builtins.insert("eval".to_string(), eval);
        self.builtins.insert("exec".to_string(), exec::exec);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("export".to_string(), parameter::export);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("fg".to_string(), job_commands::fg);
        self.builtins.insert("getopts".to_string(), getopts::getopts);
        self.builtins.insert("hash".to_string(), hash::hash);
        self.builtins.insert("history".to_string(), history::history);
        self.builtins.insert("jobs".to_string(), job_commands::jobs);
        self.builtins.insert("kill".to_string(), job_commands::kill);
        self.builtins.insert("let".to_string(), let_);
        self.builtins.insert("local".to_string(), parameter::local);
        self.builtins.insert("printf".to_string(), printf::printf);
        self.builtins.insert("pwd".to_string(), pwd::pwd);
        self.builtins.insert("read".to_string(), read::read);
        self.builtins.insert("return".to_string(), loop_control::return_);
        self.builtins.insert("set".to_string(), option::set);
        self.builtins.insert("trap".to_string(), trap::trap);
        self.builtins.insert("type".to_string(), type_::type_);
        self.builtins.insert("shift".to_string(), option::shift);
        self.builtins.insert("shopt".to_string(), option::shopt);
        self.builtins.insert("unalias".to_string(), alias::unalias);
        self.builtins.insert("unset".to_string(), unset::unset);
        self.builtins.insert("source".to_string(), source::source);
        self.builtins.insert(".".to_string(), source::source);
        self.builtins.insert("true".to_string(), true_);
        self.builtins.insert("wait".to_string(), job_commands::wait);

        self.substitution_builtins.insert("readonly".to_string(), parameter::readonly);
        self.substitution_builtins.insert("typeset".to_string(), parameter::declare);
        self.substitution_builtins.insert("declare".to_string(), parameter::declare);
    }
}

pub fn eval(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    args.remove(0);
    if args.len() > 0 && args[0] == "--" {
        args.remove(0);
    }

    let mut feeder = Feeder::new(&args.join(" "));

    core.eval_level += 1;
    match Script::parse(&mut feeder, core, false){
        Ok(Some(mut s)) => {
            let _ = s.exec(core);
        },
        Err(e) => e.print(core),
        _        => {},
    }

    core.eval_level -= 1;
    core.db.exit_status
}

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if core.db.flags.contains('i') {
        eprintln!("exit");
    }
    if args.len() > 1 {
        match &args[1].parse::<i32>() {
            Ok(n) => core.db.exit_status = *n,
            _ => core.db.exit_status = 1,
        }
    }
    exit::normal(core)
}

pub fn false_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    1
}

pub fn true_(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    0
}

pub fn bind(_: &mut ShellCore, _: &mut Vec<String>) -> i32 {
    0
}

pub fn debug(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    dbg!("{:?}", &args);
    dbg!("{:?}", &core.db.get_param("depth"));
    0
}

pub fn let_(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut last_result = 0;
    for a in &args[1..] {
        match ArithmeticExpr::parse(&mut Feeder::new(&a.replace("$", "\\$")), core, false, "") {
            Ok(Some(mut a)) => {
                match a.eval(core) {
                    Ok(s) => last_result = if s == "0" {1} else {0},
                    Err(e) => {
                        return error_exit(1, &args[0], &String::from(&e), core);
                    },
                }
            },
            Ok(None) => {
                return error_exit(1, &args[0], "expression expected", core);
            },
            Err(e) => {
                return error_exit(1, &args[0], &String::from(&e), core);
            }
        }
    }

    last_result
}
