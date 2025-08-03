//SPDX-FileCopyrightText: 2025 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::command::simple::SimpleCommand;
use crate::elements::io::pipe::Pipe;
use crate::utils::{arg, file};
use crate::{error, proc_ctrl, ShellCore};

pub fn builtin(core: &mut ShellCore, args: &[String]) -> i32 {
    if args.len() <= 1 {
        return 0;
    }

    if !core.builtins.contains_key(&args[1]) {
        let msg = format!("{}: not a shell builtin", &args[1]);
        return super::error_exit(1, &args[0], &msg, core);
    }

    core.builtins[&args[1]](core, &args[1..])
}

fn command_v(words: &[String], core: &mut ShellCore, large_v: bool) -> i32 {
    if words.is_empty() {
        return 0;
    }

    let mut return_value = 1;

    for com in words.iter() {
        if core.aliases.contains_key(com) {
            match large_v {
                true => println!("{} is aliased to `{}'", &com, core.aliases[com]),
                false => println!("alias {}='{}'", &com, &core.aliases[com]),
            }
        } else if core.builtins.contains_key(com) {
            return_value = 0;

            match large_v {
                true => println!("{} is a shell builtin", &com),
                false => println!("{}", &com),
            }
        } else if let Some(path) = file::search_command(com) {
            return_value = 0;
            match large_v {
                true => println!("{} is {}", &com, &path),
                false => println!("{}", &com),
            }
        } else if large_v {
            let msg = format!("command: {com}: not found");
            error::print(&msg, core);
        }
    }

    return_value
}

pub fn command(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = arg::dissolve_options(&args.to_vec());
    if core.db.flags.contains('r') && arg::consume_option("-p", &mut args) {
        return super::error_exit(1, &args[0], "-p: restricted", core);
    }

    if args.len() <= 1 {
        return 0;
    }

    let mut pos = 1;
    while args.len() > pos {
        match args[pos].starts_with("-") {
            true => pos += 1,
            false => break,
        }
    }

    let words = args[pos..].to_vec();
    if words.is_empty() {
        return 0;
    }

    let mut args = args[..pos].to_vec();
    args = arg::dissolve_options(&args);

    let last_option = args.last().unwrap();
    if last_option == "-V" || last_option == "-v" {
        return command_v(&words, core, last_option == "-V");
    } else if core.builtins.contains_key(&words[0]) {
        return core.builtins[&words[0]](core, &words);
    }

    let mut command = SimpleCommand::default();
    let mut pipe = Pipe::new("".to_string());
    command.args = words;
    if let Ok(pid) = command.exec_command(core, &mut pipe) {
        proc_ctrl::wait_pipeline(core, vec![pid], false, false);
    }

    core.db.exit_status
}
