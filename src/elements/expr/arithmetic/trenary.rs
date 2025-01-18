//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::exec::ExecError;
use super::{ArithmeticExpr, ArithElem};
use super::calculator;

pub fn operation(left: &Option<ArithmeticExpr>, right: &Option<ArithmeticExpr>,
    stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), ExecError> {
    let num = calculator::pop_operand(stack, core)?;

    let mut left = match left {
        Some(c) => c.clone(),
        None    => return Err(ExecError::Other("expr not found".to_string())),
    };
    let mut right = match right {
        Some(c) => c.clone(),
        None    => return Err(ExecError::Other("expr not found".to_string())),
    };

    let ans = match num {
        ArithElem::Integer(0) => right.eval_in_cond(core)?,
        ArithElem::Float(_) => return Err(ExecError::Other("float condition is not permitted".to_string())),
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
