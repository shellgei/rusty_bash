//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::utils::exit;
use crate::core::data::Value;
use crate::elements::substitution::Substitution;
use crate::utils::option;

pub fn set_positions(core: &mut ShellCore, args: &[String]) -> i32 {
    match core.data.position_parameters.pop() {
        None => exit::internal("empty param stack"),
        _    => {},
    }
    core.data.position_parameters.push(args.to_vec());
    core.data.set_param("#", &(args.len()-1).to_string());
    0
}

fn print_data(k: &str, core: &mut ShellCore) {
    match core.data.get_value(k) {
        Some(Value::EvaluatedSingle(s)) => {
            println!("{}={}", k.to_string(), s.to_string()); 
        },
        Some(Value::EvaluatedArray(a)) => {
            let mut formatted = String::new();
            formatted += "(";
            for (i, v) in a.iter().enumerate() {
                formatted += &format!("[{}]=\"{}\" ", i, v).clone();
            }
            if formatted.ends_with(" ") {
                formatted.pop();
            }
            formatted += ")";
            println!("{}={}", k.to_string(), formatted); 
        },
        _ => {},
    }
}

pub fn print_all(core: &mut ShellCore) -> i32 {
    core.data.get_keys()
        .into_iter()
        .for_each(|k| print_data(&k, core));
    0
}

fn set_local(arg: &str, core: &mut ShellCore, layer: usize) -> bool {
    let mut feeder = Feeder::new(arg);
    if feeder.scanner_name(core) == feeder.len() { // name only
        let name = feeder.consume(feeder.len());
        return core.data.set_layer_param(&name, "", layer);
    }

    let mut sub = match Substitution::parse(&mut feeder, core) {
        Some(s) => s,
        _ => {
            eprintln!("sush: local: `{}': not a valid identifier", arg);
            return false;
        },
    };

    match sub.eval(core) {
        true => {},
        false => exit::internal("unsupported substitution"),
    }

    match sub.evaluated_value {
        Value::EvaluatedSingle(s) => core.data.set_layer_param(&sub.key, &s, layer),
        Value::EvaluatedArray(a)  => core.data.set_layer_array(&sub.key, &a, layer),
        _ => exit::internal("unsupported substitution"),
    }
}

pub fn local(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let layer = if core.data.get_layer_num() > 2 {
        core.data.get_layer_num() - 2 //The last element of data.parameters is for local itself.
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

    let args = option::dissolve_options(args);
    dbg!("{:?}", &args);

    0
}
