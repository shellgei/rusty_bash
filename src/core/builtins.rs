//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

mod alias;
mod cd;
pub mod completion;
mod getopts;
mod history;
mod job_commands;
pub mod parameter;
pub mod option;
mod printf;
mod pwd;
mod read;
mod source;
mod loop_control;
mod unset;

use crate::{error, proc_ctrl, Feeder, Script, ShellCore};
use crate::error::parse;
use crate::elements::command::simple::SimpleCommand;
use crate::elements::io::pipe::Pipe;
use crate::utils::{arg, exit, file};

impl ShellCore {
    pub fn set_builtins(&mut self) {
        self.builtins.insert(":".to_string(), true_);
        self.builtins.insert("alias".to_string(), alias::alias);
        self.builtins.insert("bg".to_string(), job_commands::bg);
        self.builtins.insert("break".to_string(), loop_control::break_);
        self.builtins.insert("builtin".to_string(), builtin);
        self.builtins.insert("cd".to_string(), cd::cd);
        self.builtins.insert("command".to_string(), command);
        self.builtins.insert("compgen".to_string(), completion::compgen);
        self.builtins.insert("complete".to_string(), completion::complete);
        self.builtins.insert("continue".to_string(), loop_control::continue_);
        self.builtins.insert("declare".to_string(), parameter::declare);
        self.builtins.insert("eval".to_string(), eval);
        self.builtins.insert("exit".to_string(), exit);
        self.builtins.insert("false".to_string(), false_);
        self.builtins.insert("fg".to_string(), job_commands::fg);
        self.builtins.insert("getopts".to_string(), getopts::getopts);
        self.builtins.insert("history".to_string(), history::history);
        self.builtins.insert("jobs".to_string(), job_commands::jobs);
        self.builtins.insert("local".to_string(), parameter::local);
        self.builtins.insert("printf".to_string(), printf::printf);
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

pub fn command_v(words: &mut Vec<String>, core: &mut ShellCore, large_v: bool) -> i32 {
    if words.is_empty() {
        return 0;
    }

    let mut return_value = 1;

    for com in words.iter() {
        if core.aliases.contains_key(com) {
            match large_v {
                true  => println!("{} is aliased to `{}'", &com, core.aliases[com]),
                false => println!("alias {}='{}'", &com, &core.aliases[com]),
            }
        }else if core.builtins.contains_key(com) {
            return_value = 0;

            match large_v {
                true  => println!("{} is a shell builtin", &com),
                false => println!("{}", &com),
            }
        }else if let Some(path) = file::search_command(&com) {
            return_value = 0;
            match large_v {
                true  => println!("{} is {}", &com, &path),
                false => println!("{}", &com),
            }
        }else if large_v {
            let msg = format!("command: {}: not found", com);
            error::print(&msg, core);
        }
    }

    return_value
}

pub fn command(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return 0;
    }

    let mut args = arg::dissolve_options(args);
    let mut words = arg::consume_after_options(&mut args, 1);

    if words.is_empty() {
        return 0;
    }

    let last_option = args.last().unwrap();
    if last_option == "-V" {
        return command_v(&mut words, core, true);
    }else if last_option == "-v" {
        return command_v(&mut words, core, false);
    }

    if core.builtins.contains_key(&words[0]) {
        return core.builtins[&words[0]](core, &mut words);
    }

    let mut command = SimpleCommand::default();
    let mut pipe = Pipe::new("".to_string());
    command.args = words;
    if let Ok(pid) = command.exec_command(core, &mut pipe) {
        proc_ctrl::wait_pipeline(core, vec![pid], false, false);
    }

    core.db.exit_status
}

pub fn eval(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let args = arg::consume_after_options(args, 1);
    let mut feeder = Feeder::new(&args.join(" "));

    core.eval_level += 1;
    match Script::parse(&mut feeder, core, false){
        Ok(Some(mut s)) => {let _ = s.exec(core); },
        Err(e) => parse::print_error(e, core),
        _        => {},
    }

    core.eval_level -= 1;
    core.db.exit_status
}

pub fn exit(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    eprintln!("exit");
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
