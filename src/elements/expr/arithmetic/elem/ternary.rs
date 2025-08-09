//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::super::calculator;
use super::super::{ArithElem, ArithmeticExpr};
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::ShellCore;

pub fn operation(
    left: &Option<ArithmeticExpr>,
    right: &Option<ArithmeticExpr>,
    stack: &mut Vec<ArithElem>,
    core: &mut ShellCore,
) -> Result<(), ExecError> {
    if left.is_none() {
        return Err(ArithError::ExpressionExpected(":".to_string()).into());
    }
    let mut left = left.clone().unwrap();

    if right.is_none() {
        return Err(ArithError::NoColon(left.text.trim().to_string()).into());
    }

    let mut right = right.clone().unwrap();

    if left.elements.is_empty() {
        let msg = format!(":{}", &right.text.trim_end());
        return Err(ArithError::ExpressionExpected(msg).into());
    }

    if right.elements.is_empty() {
        return Err(ArithError::ExpressionExpected(":".to_string()).into());
    }

    let ans = match calculator::pop_operand(stack, core)? {
        ArithElem::Integer(0) => right.eval_in_cond(core)?,
        ArithElem::Float(_) => {
            return Err(ExecError::Other(
                "float condition is not permitted".to_string(),
            ))
        }
        _ => left.eval_in_cond(core)?,
    };

    stack.push(ans);
    Ok(())
}
