//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error, ShellCore};
use super::{ArithElem, word};

pub fn unary_calc(op: &str, num: f64, stack: &mut Vec<ArithElem>) -> Result<(), String> {
    match op {
        "+"  => stack.push( ArithElem::Float(num) ),
        "-"  => stack.push( ArithElem::Float(-num) ),
        _ => return Err("not supported operator for float number".to_string()),
    }
    Ok(())
}

pub fn bin_calc(op: &str, left: f64, right: f64,
                stack: &mut Vec<ArithElem>) -> Result<(), String> {
    let bool_to_01 = |b| { if b { ArithElem::Integer(1) } else { ArithElem::Integer(0) } };

    match op {
        "+"  => stack.push(ArithElem::Float(left + right)),
        "-"  => stack.push(ArithElem::Float(left - right)),
        "*"  => stack.push(ArithElem::Float(left * right)),
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
            stack.push(ArithElem::Float(left / right));
        },
        "**" => {
            if right >= 0.0 {
                let r = right.try_into().unwrap();
                stack.push(ArithElem::Float(left.powf(r)));
            }else{
                return Err( error::exponent(&right.to_string()) );
            }
        },
        _    => return Err("not supported operator for float numbers".to_string()),
    }

    Ok(())
}

pub fn substitute(op: &str, name: &String, cur: f64, right: f64, core: &mut ShellCore)
                                      -> Result<ArithElem, String> {
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

    match core.db.set_param(&name, &new_value.to_string(), None) { //TODO: simplify, None, None
        Ok(()) => Ok(ArithElem::Float(new_value)),
        Err(e) => Err(e),
    }
}

pub fn parse(s: &str) -> Result<f64, String> {
    let mut sw = s.to_string();
    let sign = word::get_sign(&mut sw);

    match (sw.parse::<f64>(), sign.as_str()) {
        (Ok(f), "-") => Ok(-f),
        (Ok(f), _)   => Ok(f),
        (Err(e), _)  => Err(e.to_string()),
    }
}
