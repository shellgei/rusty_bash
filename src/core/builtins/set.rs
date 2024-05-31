//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::data::Value;
use std::collections::HashMap;

fn set_no_arg(core: &mut ShellCore) -> i32 {
    let mut output = HashMap::new();
    for k in core.data.get_keys() {
        match core.data.get_value(&k) {
                Some(Value::EvaluatedSingle(s)) => {output.insert(k.to_string(), s.to_string());},
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
                    output.insert(k.to_string(), formatted);
                },
                _ => {},
        }
    }

    /*
    for layer in &core.data.parameters {
        for e in layer {
            match e.1 {
                Value::EvaluatedSingle(s) => {output.insert(e.0.to_string(), s.to_string());},
                Value::EvaluatedArray(a) => {
                    let mut formatted = String::new();
                    formatted += "(";
                    for (i, v) in a.iter().enumerate() {
                        formatted += &format!("[{}]=\"{}\" ", i, v).clone();
                    }
                    if formatted.ends_with(" ") {
                        formatted.pop();
                    }
                    formatted += ")";
                    output.insert(e.0.to_string(), formatted);
                },
                _ => {},
        }
        }
    }
    */

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
