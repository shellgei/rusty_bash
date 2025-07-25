//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use super::SimpleCommand;
use super::SubsArgType;
use crate::error::exec::ExecError;
use crate::elements::substitution::Substitution;

pub fn run(com: &mut SimpleCommand, core: &mut ShellCore)
-> Result<bool, ExecError> {
    let ans = run_function(&mut com.args, core) 
           || run_substitution_builtin(com, core)?
           || run_builtin(com, core)?;
    Ok(ans)
}

fn run_function(args: &mut Vec<String>, core: &mut ShellCore) -> bool {
    match core.db.functions.get_mut(&args[0]) {
        Some(f) => {f.clone().run_as_command(args, core); true},
        None => false,
    }
}

pub fn run_builtin(com: &mut SimpleCommand, core: &mut ShellCore)
-> Result<bool, ExecError> {
    if com.args.is_empty() {
        eprintln!("ShellCore::run_builtin");
        return Ok(false);
    }

    if ! core.builtins.contains_key(&com.args[0]) {
        return Ok(false);
    }

    let func = core.builtins[&com.args[0]];
    core.db.exit_status = func(core, &mut com.args);
    Ok(true)
}

pub fn run_substitution_builtin(com: &mut SimpleCommand, core: &mut ShellCore)
-> Result<bool, ExecError> {
    if com.args.is_empty() {
        eprintln!("ShellCore::run_builtin");
        return Ok(false);
    }

    if ! core.substitution_builtins.contains_key(&com.args[0]) {
        return Ok(false);
    }

    let func = core.substitution_builtins[&com.args[0]];
    let mut subs = vec![];
    for sub in com.substitutions_as_args.iter_mut() {
        match sub {
            SubsArgType::Subs(s) => subs.push(s.clone()),
            SubsArgType::Other(w) => {
                for arg in w.eval(core)? {
                    if arg.starts_with("-") || arg.starts_with("+") {
                        com.args.push(arg);
                    }
                    else {
                        let mut f = Feeder::new(&arg);
                        match Substitution::parse(&mut f, core, true)? {
                            Some(s) => subs.push(s),
                            _ => {
                                let mut s = Substitution::default();
                                s.text = arg;
                                s.left_hand.text = s.text.clone();
                                s.left_hand.name = s.text.clone();
                                subs.push(s);
                            },
                        }
                    }
                }
            },
        }
    }

    core.db.exit_status = func(core, &mut com.args, &mut subs);
    Ok(true)
}
