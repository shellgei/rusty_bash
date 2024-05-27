//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::substitution::{Substitution, Value};

fn parse(arg: &str, core: &mut ShellCore) -> Option<Substitution> {
    let mut feeder = Feeder::new();
    feeder.add_line(arg.to_string());

    match Substitution::parse(&mut feeder, core) {
        Some(sub) => Some(sub),
        _ => {
            eprintln!("sush: local: `{}': not a valid identifier", arg);
            None
        },
    }
}

pub fn local(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    let layer = if core.data.parameters.len() > 2 {
        core.data.parameters.len() - 2 //The last element of data.parameters is for local itself.
    }else{
        eprintln!("sush: local: can only be used in a function");
        return 1;
    };

    for arg in &args[1..] {
        let mut sub = match parse(arg, core) {
            Some(s) => s,
            None    => return 1,
        };

        match sub.eval(core) {
            Value::EvaluatedSingle(s) => core.data.set_layer_param(&sub.key, &s, layer),
            Value::EvaluatedArray(a)  => core.data.set_layer_array(&sub.key, &a, layer),
            _ => panic!("SUSH INTERNAL ERROR: unsupported substitution"),
        }
    }

    0
}
