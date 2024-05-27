//SPDX-FileCopyrightText: 2023 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-FileCopyrightText: 2023 @caro@mi.shellgei.org
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::substitution::{Substitution, Value};

pub fn local(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    if core.data.parameters.len() <= 2 {
        eprintln!("sush: local: can only be used in a function");
        return 1;
    }

    let layer = core.data.parameters.len() - 2; //The last element of data.parameters is for local itself.

    for arg in &args[1..] {
        let mut feeder = Feeder::new();
        feeder.add_line(arg.clone());
        match Substitution::parse(&mut feeder, core) {
            Some(mut sub) => {
                match sub.eval(core) {
                    Value::EvaluatedSingle(s) => {
                        core.data.parameters[layer].insert(sub.key.to_string(), s);
                    },
                    Value::EvaluatedArray(a) => {
                        core.data.arrays[layer].insert(sub.key.to_string(), a);
                    },
                    _ => {
                        eprintln!("sush: local: `{}': not a valid identifier", arg);
                        return 1;
                    },
                }
            },
            _ => {
                eprintln!("sush: local: `{}': not a valid identifier", arg);
                return 1;
            },
        }
    }

    0
}
