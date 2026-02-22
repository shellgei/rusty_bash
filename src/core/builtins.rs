//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod alias;
mod caller;
mod cd;
mod command;
pub mod compgen;
pub mod complete;
mod compopt;
mod echo;
mod exec;
mod getopts;
mod hash;
mod history;
mod job_commands;
mod loop_control;
pub mod option;
pub mod variable;
mod printf;
mod pwd;
mod read;
pub mod source;
mod trap;
mod type_;
#[cfg(not(target_os = "macos"))]
mod ulimit;
mod unset;

use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{exit, Feeder, Script, ShellCore};
use std::process::Command;
//#[cfg(not(target_os = "macos"))]

pub fn error_(exit_status: i32, name: &str, msg: &str, core: &mut ShellCore) -> i32 {
    let shellname = core.db.get_param("0").unwrap();
    if core.db.flags.contains('i') {
        eprintln!("{}: {}: {}", &shellname, name, msg);
    } else {
        let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
        eprintln!("{}: line {}: {}: {}", &shellname, &lineno, name, msg);
    }
    exit_status
}

pub fn error(exit_status: i32, name: &str, err: &ExecError, core: &mut ShellCore) -> i32 {
    error_(exit_status, name, &String::from(err), core)
}

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("alias".to_string(), alias::alias);
        self.builtins.insert("bg".to_string(), job_commands::bg);
        self.builtins.insert("bind".to_string(), bind);
        self.builtins
            .insert("break".to_string(), loop_control::break_);
        self.builtins
            .insert("builtin".to_string(), command::builtin);
        self.builtins.insert("cd".to_string(), cd::cd);
        self.builtins.insert("caller".to_string(), caller::caller);
        self.builtins
            .insert("command".to_string(), command::command);
        self.builtins
            .insert("compgen".to_string(), compgen::compgen);
        self.builtins
            .insert("complete".to_string(), complete::complete);
        self.builtins
            .insert("compopt".to_string(), compopt::compopt);
        self.builtins
            .insert("continue".to_string(), loop_control::continue_);
        self.builtins.insert("debug".to_string(), debug);
        self.builtins
            .insert("disown".to_string(), job_commands::disown);
        self.builtins.insert("echo".to_string(), echo::echo);
        self.builtins.insert("eval".to_string(), eval);
        self.builtins.insert("exec".to_string(), exec::exec);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("fg".to_string(), job_commands::fg);
        self.builtins
            .insert("getopts".to_string(), getopts::getopts);
        self.builtins.insert("hash".to_string(), hash::hash);
        self.builtins
            .insert("history".to_string(), history::history);
        self.builtins.insert("jobs".to_string(), job_commands::jobs);
        self.builtins.insert("kill".to_string(), job_commands::kill);
        self.builtins.insert("let".to_string(), let_);
        self.builtins.insert("printf".to_string(), printf::printf);
        self.builtins.insert("pwd".to_string(), pwd::pwd);
        self.builtins.insert("read".to_string(), read::read);
        self.builtins
            .insert("return".to_string(), loop_control::return_);
        self.builtins.insert("set".to_string(), option::set);
        self.builtins.insert("trap".to_string(), trap::trap);
        self.builtins.insert("type".to_string(), type_::type_);
        self.builtins.insert("shift".to_string(), option::shift);
        self.builtins.insert("shopt".to_string(), option::shopt);

        //if file::search_command("ulimit").is_none() {
        #[cfg(not(target_os = "macos"))]
        self.builtins.insert("ulimit".to_string(), ulimit::ulimit);
        /*
        #[cfg(target_os = "macos")]
                    self.builtins.insert("ulimit".to_string(), ulimit_mac::ulimit);
                }*/

        self.builtins.insert("unalias".to_string(), alias::unalias);
        self.builtins.insert("unset".to_string(), unset::unset);
        self.builtins.insert("source".to_string(), source::source);
        self.builtins.insert(".".to_string(), source::source);
        self.builtins.insert("true".to_string(), true_);
        self.builtins.insert("test".to_string(), test);
        self.builtins.insert("[".to_string(), test);
        self.builtins.insert("wait".to_string(), job_commands::wait);

        self.subst_builtins
            .insert("export".to_string(), variable::export);
        self.subst_builtins
            .insert("readonly".to_string(), variable::readonly);
        self.subst_builtins
            .insert("typeset".to_string(), variable::declare);
        self.subst_builtins
            .insert("declare".to_string(), variable::declare);
        self.subst_builtins
            .insert("local".to_string(), variable::local);
    }
}

pub fn eval(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = args.to_owned();
    args.remove(0);
    if !args.is_empty() && args[0] == "--" {
        args.remove(0);
    }

    let script = args.join(" ");
    let mut feeder = Feeder::new(&script);
    let lineno = match core.db.get_param("LINENO") {
        Ok(s) => s.parse::<usize>().unwrap_or_default(),
        _ => 0,
    };
    feeder.lineno += lineno - 1;

    match Script::parse(&mut feeder, core, false) {
        Ok(Some(mut s)) => {
            core.eval_level += 1;
            let _ = s.exec(core);
            core.eval_level -= 1;
        }
        Err(ParseError::UnexpectedSymbol(t)) => {
            let lineno = core.db.get_param("LINENO").unwrap_or("0".to_string());
            let com = &core.db.position_parameters[0][0];
            eprintln!(
                "{}: eval: line {}: syntax error near unexpected token `{}'",
                com, &lineno, &t
            );
            eprintln!("{}: eval: line {}: `{}'", com, &lineno, &script);
            return 2;
        }
        Err(e) => e.print(core),
        _ => {}
    }

    core.db.exit_status
}

pub fn exit(core: &mut ShellCore, args: &[String]) -> i32 {
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

pub fn false_(_: &mut ShellCore, _: &[String]) -> i32 {
    1
}

pub fn true_(_: &mut ShellCore, _: &[String]) -> i32 {
    0
}

pub fn bind(_: &mut ShellCore, _: &[String]) -> i32 {
    0
}

pub fn debug(_: &mut ShellCore, _: &[String]) -> i32 {
//    let pos = core.db.get_scope_pos("words").unwrap();
//
//    dbg!("{:?}", &core.db.params.len());
//    dbg!("{:?}", &pos);
//    dbg!("{:?}", &core.db.params[pos].get("words"));
    0
}

pub fn let_(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut last_result = 0;
    core.valid_assoc_expand_once = true;

    for a in &args[1..] {
        match ArithmeticExpr::parse(&mut Feeder::new(&a.replace("$", "\\$")), core, false, "") {
            Ok(Some(mut a)) => match a.eval(core) {
                Ok(s) => last_result = if s == "0" { 1 } else { 0 },
                Err(e) => {
                    core.valid_assoc_expand_once = false;
                    return error(1, &args[0], &e, core);
                }
            },
            Ok(None) => {
                core.valid_assoc_expand_once = false;
                return error_(1, &args[0], "expression expected", core);
            }
            Err(e) => {
                core.valid_assoc_expand_once = false;
                return error(1, &args[0], &From::from(e), core);
            }
        }
    }

    core.valid_assoc_expand_once = false;
    last_result
}

pub fn test(core: &mut ShellCore, args: &[String]) -> i32 {
    /* difference between the builtin test and the external command */
    if (args.len() == 5 && args[0] == "[" && args[4] == "]")
    || (args.len() == 4 && args[0] == "test") {
        if args[2] == "=" {
            if args[1] == args[3] {
                return 0;
            }else if args[1] != args[3] {
                return 1;
            }
        }
    }

    /* call the external test command */
    match Command::new(&args[0]).args(args[1..].to_vec()).output() {
        Ok(com) => {
            let exit_status = com.status.code().unwrap_or(127);
            if exit_status > 1 {
                let msg = String::from_utf8(com.stderr).unwrap_or("".to_string());
                let shellname = core.db.get_param("0").unwrap();
                if core.db.flags.contains('i') {
                    eprintln!("{}: {}", &shellname, msg);
                } else {
                    let lineno = core.db.get_param("LINENO").unwrap_or("".to_string());
                    eprintln!("{}: line {}: {}", &shellname, &lineno, msg);
                }
            }
            exit_status
        },
        _ => 127
    }
}
