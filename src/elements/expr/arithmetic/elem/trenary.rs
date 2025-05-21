//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use super::super::{ArithmeticExpr, ArithElem};
use super::super::calculator;

pub fn operation(left: &Option<ArithmeticExpr>, right: &Option<ArithmeticExpr>,
    stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), ExecError> {

    let e = ExecError::Other("expr not found".to_string());
    let mut left = left.clone().ok_or(e.clone())?;
    let mut right = right.clone().ok_or(e.clone())?;

    if left.elements.is_empty() || right.elements.is_empty() {
        let msg = format!(": {}", &right.text);
        return Err(ArithError::OperandExpected(msg).into());
    }

    let ans = match calculator::pop_operand(stack, core)? {
        ArithElem::Integer(0, _) => right.eval_in_cond(core)?,
        ArithElem::Float(_, _) => return Err(ExecError::Other("float condition is not permitted".to_string())),
        _ => left.eval_in_cond(core)?,
    };

    stack.push( ans );
    Ok(())
}
