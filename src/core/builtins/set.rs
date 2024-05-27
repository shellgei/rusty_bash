//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use std::collections::HashMap;

fn set_no_arg(core: &mut ShellCore) -> i32 {
    let mut output = HashMap::new();
    for layer in &core.data.parameters {
        layer.iter().for_each(|e| {output.insert(e.0, e.1); } );
    }

    for e in output {
        println!("{}={}", e.0, e.1); 
    }
    0
}

pub fn set_parameters(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    match core.data.position_parameters.pop() {
        None => panic!("SUSH INTERNAL ERROR: empty param stack"),
        _    => {},
    }
    core.data.position_parameters.push(args.to_vec());
    0
}

pub fn set(core: &mut ShellCore, args: &mut Vec<String>) -> i32 {
    match args.len() {
        0 => panic!("never come here"),
        1 => set_no_arg(core),
        _ => set_parameters(core, args),
    }
}
