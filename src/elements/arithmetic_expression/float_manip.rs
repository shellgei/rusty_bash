//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Elem;
use super::calculator::exponent_error_msg;

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
                return Err( exponent_error_msg(&right.to_string()) );
            }
        },
        _    => return Err("not supported operator for float numbers".to_string()),
    }

    Ok(())
}
