//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::{Elem, error_msg, word};

pub fn unary_calc(op: &str, num: f64, stack: &mut Vec<Elem>) -> Result<(), String> {
    match op {
        "+"  => stack.push( Elem::Float(num) ),
        "-"  => stack.push( Elem::Float(-num) ),
        _ => return Err("not supported operator for float number".to_string()),
    }
    Ok(())
}

pub fn bin_calc(op: &str, left: f64, right: f64,
                stack: &mut Vec<Elem>) -> Result<(), String> {
    let bool_to_01 = |b| { if b { Elem::Integer(1) } else { Elem::Integer(0) } };

    match op {
        "+"  => stack.push(Elem::Float(left + right)),
        "-"  => stack.push(Elem::Float(left - right)),
        "*"  => stack.push(Elem::Float(left * right)),
        "<="  => stack.push(bool_to_01( left <= right )),
        ">="  => stack.push(bool_to_01( left >= right )),
        "<"  => stack.push(bool_to_01( left < right )),
        ">"  => stack.push(bool_to_01( left > right )),
        "=="  => stack.push(bool_to_01( left == right )),
        "!="  => stack.push(bool_to_01( left != right )),
        "/" => {
            if right == 0.0 {
                return Err("divided by 0".to_string());
            }
            stack.push(Elem::Float(left / right));
        },
        "**" => {
            if right >= 0.0 {
                let r = right.try_into().unwrap();
                stack.push(Elem::Float(left.powf(r)));
            }else{
                return Err( error_msg::exponent(&right.to_string()) );
            }
        },
        _    => return Err("not supported operator for float numbers".to_string()),
    }

    Ok(())
}

pub fn substitute(op: &str, name: &String, cur: f64, right: f64, core: &mut ShellCore)
                                      -> Result<Elem, String> {
    let new_value = match op {
        "+=" => cur + right,
        "-=" => cur - right,
        "*=" => cur * right,
        "/=" => {
            match right == 0.0 {
                true  => return Err("divided by 0".to_string()),
                false => cur / right,
            }
        },
        _   => return Err("Not supprted operation for float numbers".to_string()),
    };

    core.data.set_param(&name, &new_value.to_string());
    Ok(Elem::Float(new_value))
}

pub fn parse(s: &str) -> Option<f64> {
    let mut sw = s.to_string();
    let sign = word::get_sign(&mut sw);

    match (sw.parse::<f64>(), sign.as_str()) {
        (Ok(f), "-") => Some(-f),
        (Ok(f), _)   => Some(f),
        _            => None,
    }
}
