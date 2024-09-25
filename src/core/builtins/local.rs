//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error, ShellCore, Feeder};
use crate::core::data::Value;
use crate::elements::substitution::Substitution;

fn set(arg: &str, core: &mut ShellCore, layer: usize) -> bool {
    let mut sub = match Substitution::parse(&mut Feeder::new(arg), core) {
        Some(s) => s,
        _ => {
            eprintln!("sush: local: `{}': not a valid identifier", arg);
            return false;
        },
    };

    match sub.eval(core) {
        Value::EvaluatedSingle(s) => core.data.set_layer_param(&sub.key, &s, layer),
        Value::EvaluatedArray(a)  => core.data.set_layer_array(&sub.key, &a, layer),
        _ => error::internal("unsupported substitution"),
    }
    true
}

pub fn local(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let layer = if core.data.get_layer_num() > 2 {
        core.data.get_layer_num() - 2 //The last element of data.parameters is for local itself.
    }else{
        eprintln!("sush: local: can only be used in a function");
        return 1;
    };

    match args[1..].iter().all(|a| set(a, core, layer)) {
        true  => 0,
        false => 1,
    }
}
