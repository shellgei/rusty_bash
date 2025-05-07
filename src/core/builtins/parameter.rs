//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, utils, Feeder};
use crate::error::exec::ExecError;
use crate::elements::substitution::Substitution;
use crate::utils::arg;
use super::error_exit;

pub fn set_positions(core: &mut ShellCore, args: &[String]) -> Result<(), ExecError> {
    if core.db.position_parameters.pop().is_none() {
        return Err(ExecError::Other("empty param stack".to_string()));
    }
    core.db.position_parameters.push(args.to_vec());
    Ok(())
}

pub fn print_data(name: &str, core: &mut ShellCore) {
    core.db.print(name);
}

pub fn print_all(core: &mut ShellCore) -> i32 {
    core.db.get_keys()
        .into_iter()
        .for_each(|k| print_data(&k, core));
    0
}

fn set_local(arg: &str, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
    let mut feeder = Feeder::new(arg);
    if feeder.scanner_name(core) == feeder.len() { // name only
        let name = feeder.consume(feeder.len());

        if ! core.db.has_value_layer(&name, layer) {
            return core.db.set_param(&name, "", Some(layer));
        }else{
            return Ok(());
        }
    }

    match Substitution::parse(&mut feeder, core) {
        Ok(ans) => ans.unwrap().eval(core, Some(layer), false),
        Err(e) => Err(ExecError::ParseError(e)),
    }
}

fn set_local_array(arg: &str, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
    let mut feeder = Feeder::new(arg);
    if feeder.scanner_name(core) == feeder.len() { // name only
        let name = feeder.consume(feeder.len());
        return core.db.set_array(&name, vec![], Some(layer));
    }

    let mut sub = match Substitution::parse(&mut feeder, core) {
        Ok(Some(s)) => s,
        _ => return Err(ExecError::VariableInvalid(arg.to_string())),
    };

    sub.eval(core, Some(layer), false)
}

fn local_(core: &mut ShellCore, args: &mut Vec<String>, layer: usize) -> Result<(), ExecError> {
    if args.len() >= 3 && args[1] == "-a" {
        for a in &args[2..] {
            set_local_array(a, core, layer)?;
        }
        return Ok(());
    }

    if args.len() >= 3 && args[1] == "-A" {
        for a in &args[2..] {
            core.db.set_assoc(a, Some(layer))?;
        }
        return Ok(());
    }

    for a in &args[1..] {
        set_local(a, core, layer)?;
    }
    Ok(())
}

pub fn local(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let layer = if core.db.get_layer_num() > 2 {
        core.db.get_layer_num() - 2//The last element of data.parameters is for local itself. 
    }else{
        ExecError::ValidOnlyInFunction("local".to_string()).print(core);
        return 1;
    };

    if let Err(e) = local_(core, args, layer) {
         e.print(core);
         return 1;
    };
    0
}

fn declare_set(core: &mut ShellCore, name_and_value: &String,
               args: &mut Vec<String>, read_only: bool) -> Result<(), ExecError> {
    let mut tmp = name_and_value.clone();
    let (mut name, value) = match name_and_value.find('=') {
        Some(n) => {
            tmp.remove(n);
            let v = tmp.split_off(n);
            let n = tmp;
            (n, v)
        },
        None => (name_and_value.to_string(), "".to_string()),
    };

    if name.contains('[') && read_only {
        name = name.split('[').next().unwrap().to_string(); 
        core.db.set_flag(&name, 'r');
        return Ok(())
    }

    if ! utils::is_name(&name, core) {
        return Err(ExecError::InvalidName(name.to_string()));
    }

    let layer = Some(core.db.get_layer_num() - 2);

    if args.contains(&"-a".to_string()) {
        let mut v = vec![];
        if ! core.db.is_array(&name)
        && ! core.db.is_assoc(&name) {
            if let Ok(s) = core.db.get_param(&name) {
                v.push(s);
            }
        }

        core.db.set_array(&name, v, layer)?;
    }else if args.contains(&"-A".to_string()) {
        core.db.set_assoc(&name, layer)?;
    }else {
        match args.contains(&"-i".to_string()) {
            false => core.db.set_param(&name, &value, layer)?,
            true  => core.db.init_as_num(&name, &value, layer)?,
        };
    }

    if read_only {
        core.db.set_flag(&name, 'r');
    }
    Ok(())
}

fn declare_print(core: &mut ShellCore, names: &[String]) -> i32 {
    for n in names {
        let mut opt = if core.db.is_assoc(&n) {
            "A"
        }else if core.db.is_array(&n) {
            "a"
        }else if core.db.has_value(&n) {
            ""
        }else{
            return error_exit(1, &n, "not found", core);
        }.to_string();

        if core.db.is_readonly(&n) {
            opt += "r";
        }

        if opt.is_empty() {
            opt += "-";
        }

        let prefix = format!("declare -{} ", opt);
        print!("{}", prefix);
        core.db.print(&n);
    }
    0
}

fn declare_print_all(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() < 2 {
        return print_all(core);
    }

    if args.len() == 2 && args[1] == "-f" {
        let mut names: Vec<String> = core.db.functions.keys().map(|k| k.to_string()).collect();
        names.sort();

        for n in names {
            core.db.functions.get_mut(&n).unwrap().pretty_print(0); 
        }
        return 0;
    }

    let mut names = core.db.get_keys();
    let mut options = String::new();

    if arg::consume_option("-a", args) {
        names.retain(|n| core.db.is_array(n));
        options += "a";
    }

    if arg::consume_option("-A", args) {
        names.retain(|n| core.db.is_assoc(n));
        options += "A";
    }

    if arg::consume_option("-r", args) {
        names.retain(|n| core.db.is_readonly(n));
        options += "r";
    }

    let prefix = format!("declare -{} ", options);
    names.into_iter().for_each(|k| {print!("{}", prefix); core.db.print(&k);});

    0
}

pub fn declare(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let mut args = arg::dissolve_options(args);

    if args[1..].iter().all(|a| a.starts_with("-")) {
        return declare_print_all(core, &mut args);
    }

    if arg::consume_option("-p", &mut args) {
        return declare_print(core, &args[1..]);
    }

    let r_flg = arg::consume_option("-r", &mut args);

    let mut name_and_values = vec![];
    while args.len() > 1 {
        let nv = args.pop().unwrap();
        if nv.starts_with("-") {
            args.push(nv);
            break;
        }

        name_and_values.push(nv);
    }

    for name_and_value in name_and_values.iter().rev() {
        if let Err(e) = declare_set(core, &name_and_value, &mut args, r_flg) {
            e.print(core);
            return 1;
        }
    }

    0
}

fn export_var(arg: &str, core: &mut ShellCore) -> Result<(), ExecError> {
    let mut feeder = Feeder::new(arg);
    if feeder.scanner_name(core) == feeder.len() { // name only
        let name = feeder.consume(feeder.len());

        if ! core.db.has_value(&name) {
            return core.db.set_param(&name, "", None);
        }else{
            return Ok(())
        }
    }

    match Substitution::parse(&mut feeder, core) {
        Ok(ans) => ans.unwrap().eval(core, None, true),
        Err(e)  => Err(ExecError::ParseError(e)),
    }
}

pub fn export(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    for arg in &args[1..] {
        if export_var(arg, core).is_err() {
            let msg = format!("parse error");
            return error_exit(1, &args[0], &msg, core);
        }
    }
    0
}

pub fn readonly(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    for name in &args[1..] {
        if name.contains('=') {
            if let Err(e) = declare_set(core, &name, &mut vec![], true) {
                e.print(core);
                return 1;
            }
            continue;
        }

        if ! utils::is_name(&name, core) {
            let msg = format!("`{}': not a valid identifier", name);
            return error_exit(1, &args[0], &msg, core);
        }
        core.db.set_flag(name, 'r');
    }
    0
}
