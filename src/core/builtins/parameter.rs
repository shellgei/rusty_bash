//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error, ShellCore, utils, Feeder};
use crate::error::ExecError;
use crate::utils::exit;
use crate::elements::substitution::Substitution;
use crate::utils::arg;

pub fn set_positions(core: &mut ShellCore, args: &[String]) -> i32 {
    match core.db.position_parameters.pop() {
        None => exit::internal("empty param stack"),
        _    => {},
    }
    core.db.position_parameters.push(args.to_vec());
    //core.db.set_param("#", &(args.len()-1).to_string());
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

    let mut sub = match Substitution::parse(&mut feeder, core) {
        Some(s) => s,
        _ => return Err(ExecError::Other(format!("local: `{}': not a valid identifier", arg))),
    };

    match sub.eval(core, Some(layer), false) {
        true  => Ok(()),
        false => Err(ExecError::Other(format!("local: `{}': evaluation error", arg))),
    }
}

fn set_local_array(arg: &str, core: &mut ShellCore, layer: usize) -> Result<(), ExecError> {
    let mut feeder = Feeder::new(arg);
    if feeder.scanner_name(core) == feeder.len() { // name only
        let name = feeder.consume(feeder.len());
        return core.db.set_array(&name, vec![], Some(layer));
    }

    let mut sub = match Substitution::parse(&mut feeder, core) {
        Some(s) => s,
        _ => {
            return Err(ExecError::Other(format!("local: `{}': not a valid identifier", arg)));
        },
    };

    match sub.eval(core, Some(layer), false) {
        true  => Ok(()),
        false => Err(ExecError::Other(format!("local: `{}': evaluation error", arg))),
    }
}

fn restore_and_return(core: &mut ShellCore, result: bool) -> i32 {
    core.db.push_local();
    if result {0} else {1} 
}

pub fn local(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let _ = core.db.pop_local();
    let layer = if core.db.get_layer_num() > 1 {
        core.db.get_layer_num() - 1 //The last element of data.parameters is for local itself.
    }else{
        eprintln!("sush: local: can only be used in a function");
        return 1;
    };

    if args.len() >= 3 && args[1] == "-a" {
        let res = args[2..].iter().all(|a| set_local_array(a, core, layer).is_ok());
        return restore_and_return(core, res);
    }

    if args.len() >= 3 && args[1] == "-A" {
        let res = args[2..].iter().all(|a| core.db.set_assoc(a, Some(layer)).is_ok());
        return restore_and_return(core, res);
    }

    let res = args[1..].iter().all(|a| set_local(a, core, layer).is_ok());
    restore_and_return(core, res)
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
            return 1; //TODO: error message
        }
        if let Err(e) = core.db.set_array(&name, vec![], None) {
            let msg = format!("{:?}", &e);
            error::print(&msg, core);
            return 1;
        }

        return 0;
    }

    if args.contains(&"-A".to_string()) {
        if ! utils::is_name(&name, core) {
            return 1; //TODO: error message
        }
        if let Err(e) = core.db.set_assoc(&name, None) {
            let msg = format!("{:?}", &e);
            error::print(&msg, core);
            return 1;
        }

        return 0;
    }

    0
}
