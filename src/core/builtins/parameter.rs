//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::utils;
use crate::utils::exit;
use crate::elements::substitution::Substitution;
use crate::utils::arg;

pub fn set_positions(core: &mut ShellCore, args: &[String]) -> i32 {
    match core.db.position_parameters.pop() {
        None => exit::internal("empty param stack"),
        _    => {},
    }
    core.db.position_parameters.push(args.to_vec());
    core.db.set_param2("#", &(args.len()-1).to_string());
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

fn set_local(arg: &str, core: &mut ShellCore, layer: usize) -> bool {
    let mut feeder = Feeder::new(arg);
    if feeder.scanner_name(core) == feeder.len() { // name only
        let name = feeder.consume(feeder.len());
        return core.db.set_layer_param(&name, "", layer);
    }

    let mut sub = match Substitution::parse(&mut feeder, core) {
        Some(s) => s,
        _ => {
            eprintln!("sush: local: `{}': not a valid identifier", arg);
            return false;
        },
    };

    /*
    match sub.eval(core) {
        true => sub.set_to_shell(core, false),
        false => exit::internal("unsupported substitution"),
    }*/
    sub.eval(core, false, false)
}

pub fn local(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let layer = if core.db.get_layer_num() > 2 {
        core.db.get_layer_num() - 2 //The last element of data.parameters is for local itself.
    }else{
        eprintln!("sush: local: can only be used in a function");
        return 1;
    };

    match args[1..].iter().all(|a| set_local(a, core, layer)) {
        true  => 0,
        false => 1,
    }
}

pub fn declare(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if args.len() <= 1 {
        return print_all(core);
    }

    let mut args = arg::dissolve_options(args);
    if args.contains(&"-A".to_string()) {
        let name = args.pop().unwrap();
        if ! utils::is_name(&name, core) {
            return 1; //TODO: error message
        }
        if ! core.db.set_assoc(&name) {
            return 1; //TODO: error message
        }
    }

    0
}
