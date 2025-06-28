//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::substitution::variable::Variable;

fn unset_all(core: &mut ShellCore, name: &str) -> i32 {
    core.db.unset(name);
    0
}

fn unset_var(core: &mut ShellCore, name: &str) -> i32 {
    core.db.unset_var(name);
    0
}

fn unset_function(core: &mut ShellCore, name: &str) -> i32 {
    core.db.unset_function(name);
    0
}

pub fn unset_one(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    match args[1].as_ref() {
        "-f" => {
            if args.len() > 2 {
                let name = args.remove(2);
                return unset_function(core, &name);
            }
        },
        "-v" => {
            if args.len() > 2 {
                let name = args.remove(2);
                return unset_var(core, &name);
            }
        },
        name => {
            let name = name.to_string();
            args.remove(1);
            if ! name.contains("[") {
                return unset_all(core, &name);
            }

            let mut f = Feeder::new(&(name.to_owned()));
            if let Ok(Some(mut sub)) = Variable::parse(&mut f, core) {
                if let Ok(Some(key)) = sub.get_index(core, false, false) {
                    let nm = sub.name.clone();
                    if let Err(e) = core.db.unset_array_elem(&nm, &key) {
                        return super::error_exit(1, &args[0], &String::from(&e), core);
                    }
                    return 0;
                }
            }

            let msg = format!("{}: invalid variable", &name);
            return super::error_exit(1, &args[0], &msg, core);
        },
    }
    0
}

pub fn unset(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut exit_status = 0;

    loop {
        if args.len() < 2 {
            break;
        }

        if (args[1] == "-v" || args[1] == "-f") && args.len() == 2 {
            break;
        }

        exit_status = unset_one(core, args);
    }
    exit_status
}
