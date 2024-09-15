//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::{ArithmeticExpr, Elem};
use super::calculator;

pub fn operation(left: &Option<ArithmeticExpr>, right: &Option<ArithmeticExpr>,
    stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    let num = match calculator::pop_operand(stack, core) {
        Ok(v)  => v,
        Err(e) => return Err(e),
    };

    let mut left = match left {
        Some(c) => c.clone(),
        None    => return Err("expr not found".to_string()),
    };
    let mut right = match right {
        Some(c) => c.clone(),
        None    => return Err("expr not found".to_string()),
    };

    let ans = match num {
        Elem::Integer(0) /*| Elem::Float(0.0)*/ => {
            match right.eval_in_cond(core) {
                Ok(num) => num,
                Err(e)  => return Err(e),
            }
        },
        Elem::Float(_) => return Err("float condition is not permitted".to_string()),
        _ => {
            match left.eval_in_cond(core) {
                Ok(num) => num,
                Err(e)  => return Err(e),
            }
        },
    };

    stack.push( ans );
    Ok(())
}
