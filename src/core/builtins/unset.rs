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

pub fn unset(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        return 0;
    }

    match args[1].as_ref() {
        "-f" => {
            if args.len() > 2 {
                return unset_function(core, &args[2]);
            }
        },
        "-v" => {
            if args.len() > 2 {
                return unset_var(core, &args[2]);
            }
        },
        name => {
            if name.contains("[") {
                let mut f = Feeder::new(&(name.to_owned()));
                if let Ok(Some(mut sub)) = Variable::parse(&mut f, core) {
                    if let Ok(Some(key)) = sub.get_index(core, false, false) {
                        let nm = sub.name.clone();
                        core.db.unset_array_elem(&nm, &key);
                        return 0;
                    }
                }

                let msg = format!("{}: invalid variable", &name);
                return super::error_exit(1, &args[0], &msg, core);
            }

            return unset_all(core, name);
        },
    }
    0
}
