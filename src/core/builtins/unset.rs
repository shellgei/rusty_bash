//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use crate::elements::expr::arithmetic::ArithmeticExpr;

fn unset_all(core: &mut ShellCore, name: &str) -> Result<i32, ExecError> {
    if ! core.shopts.query("localvar_unset") {
        core.db.unset(name, None)?;
        return Ok(0);
    }

    let mut layer = core.db.get_layer_num()-1;
    if layer <= 1 {
        core.db.unset(name, None)?;
    }else{
        layer -= 1;
        core.db.unset(name, Some(layer))?;
    }
    Ok(0)
}

fn unset_var(core: &mut ShellCore, name: &str) -> Result<i32, ExecError> {
    if ! core.shopts.query("localvar_unset") {
        core.db.unset_var(name, None)?;
        return Ok(0);
    }

    let mut layer = core.db.get_layer_num()-1;
    if layer <= 1 {
        core.db.unset_var(name, None)?;
    }else{
        layer -= 1;
        core.db.unset_var(name, Some(layer))?;
    }

    Ok(0)
}

fn unset_nameref(core: &mut ShellCore, name: &str) -> i32 {
    if ! core.shopts.query("localvar_unset") {
        let _ = core.db.unset_nameref(name, None);
        return 0;
    }

    let mut layer = core.db.get_layer_num()-1;
    if layer <= 1 {
        let _ = core.db.unset_nameref(name, None);
    }else{
        layer -= 1;
        let _ = core.db.unset_nameref(name, Some(layer));
    }

    0
}

fn unset_function(core: &mut ShellCore, name: &str) -> i32 {
    core.db.unset_function(name);
    0
}

fn unset_one(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    match args[1].as_ref() {
        "-f" => {
            if args.len() > 2 {
                let name = args.remove(2);
                return unset_function(core, &name);
            }
        }
        "-v" => {
            if args.len() > 2 {
                let name = args.remove(2);
                if let Err(e) = unset_var(core, &name) {
                    return super::error_exit(1, &args[0], &e, core);
                }else{
                    return 0;
                }
            }
        }
        "-n" => {
            if args.len() > 2 {
                let name = args.remove(2);
                return unset_nameref(core, &name);
            }
        }
        name => {
            let name = name.to_string();
            args.remove(1);
            if !name.contains("[") {
                if let Err(e) = unset_all(core, &name) {
                    return super::error_exit(1, &args[0], &e, core);
                }else{
                    return 0;
                }
            }

            let pos = name.find("[").unwrap();
            let mut name = name.clone();
            let mut index = name.split_off(pos);

            if !index.ends_with("]") {
                let msg = format!("{}: invalid variable", &name);
                return super::error_exit_text(1, &args[0], &msg, core);
            }

            index.remove(0);
            index.pop();
            let mut index = index;

            if core.db.is_array(&name) {
                if let Err(_) = index.parse::<isize>() {
                    let mut f = Feeder::new(&index);
                    match ArithmeticExpr::parse(&mut f, core, false, "[") {
                        Ok(Some(mut v)) => {
                            if !f.is_empty() {
                                let e = ExecError::ArrayIndexInvalid(index.to_string());
                                return super::error_exit(1, &args[0], &e, core);
                            }
                            if let Ok(n) = v.eval(core) {
                                index = n;
                            }
                        },
                        _ => {
                            let e = ExecError::ArrayIndexInvalid(index.to_string());
                            return super::error_exit(1, &args[0], &e, core);
                        },
                    }
                }
            }

            if let Err(e) = core.db.unset_array_elem(&name, &index) {
                return super::error_exit_text(1, &args[0], &String::from(&e), core);
            }
            return 0;
        }
    }
    0
}

pub fn unset(core: &mut ShellCore, args: &[String]) -> i32 {
    let mut args = args.to_owned();
    let mut exit_status = 0;

    loop {
        if args.len() < 2 {
            break;
        }

        if (args[1] == "-v" || args[1] == "-f" || args[1] == "-n")
        && args.len() == 2 {
            break;
        }

        exit_status = unset_one(core, &mut args);
    }
    exit_status
}
