//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::core::data::Value;

fn set_no_arg_print(k: &str, core: &mut ShellCore) {
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

fn set_no_arg(core: &mut ShellCore) -> i32 {
    core.data.get_keys()
        .into_iter()
        .for_each(|k| set_no_arg_print(&k, core));
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
