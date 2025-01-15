//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, utils, Feeder};
use crate::error::exec;
use crate::error::exec::ExecError;
use crate::utils::exit;
use crate::elements::substitution::Substitution;
use crate::utils::arg;

pub fn set_positions(core: &mut ShellCore, args: &[String]) -> i32 {
    match core.db.position_parameters.pop() {
        None => exit::internal("empty param stack"),
        _    => {},
    }
    core.db.position_parameters.push(args.to_vec());
    0
}

fn print_data(name: &str, core: &mut ShellCore) {
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
        return core.db.set_param(&name, "", Some(layer));
    }

    Substitution::parse(&mut feeder, core)?.unwrap().eval(core, Some(layer), false)
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
        exec::print_error(ExecError::ValidOnlyInFunction("local".to_string()), core);
        return 1;
    };

    if let Err(e) = local_(core, args, layer) {
         exec::print_error(e, core);
         return 1;
    };
    0
}

pub fn declare(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return print_all(core);
    }

    let mut args = arg::dissolve_options(args);

    let name = args.pop().unwrap();
    if args.contains(&"-r".to_string()) {
        core.db.set_flag(&name, 'r');
        return 0;
    }

    if args.contains(&"-a".to_string()) {
        if ! utils::is_name(&name, core) {
            let e = ExecError::InvalidName(name.to_string());
            exec::print_error(e, core);
            return 1;
        }
        if let Err(e) = core.db.set_array(&name, vec![], None) {
            exec::print_error(e, core);
            return 1;
        }

        return 0;
    }

    if args.contains(&"-A".to_string()) {
        if ! utils::is_name(&name, core) {
            let e = ExecError::InvalidName(name.to_string());
            exec::print_error(e, core);
            return 1;
        }
        if let Err(e) = core.db.set_assoc(&name, None) {
            exec::print_error(e, core);
            return 1;
        }

        return 0;
    }

    0
}
