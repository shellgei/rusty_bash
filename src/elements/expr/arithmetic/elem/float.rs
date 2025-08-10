//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::variable;
use super::ArithElem;
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::{error, ShellCore};

pub fn unary_calc(op: &str, num: f64, stack: &mut Vec<ArithElem>) -> Result<(), ExecError> {
    match op {
        "+" => stack.push(ArithElem::Float(num)),
        "-" => stack.push(ArithElem::Float(-num)),
        _ => {
            return Err(ExecError::Other(
                "not supported operator for float number".to_string(),
            ))
        }
    }
    Ok(())
}

pub fn bin_calc(
    op: &str,
    left: f64,
    right: f64,
    stack: &mut Vec<ArithElem>,
) -> Result<(), ExecError> {
    let bool_to_01 = |b| {
        if b {
            ArithElem::Integer(1)
        } else {
            ArithElem::Integer(0)
        }
    };

    match op {
        "+" => stack.push(ArithElem::Float(left + right)),
        "-" => stack.push(ArithElem::Float(left - right)),
        "*" => stack.push(ArithElem::Float(left * right)),
        "<=" => stack.push(bool_to_01(left <= right)),
        ">=" => stack.push(bool_to_01(left >= right)),
        "<" => stack.push(bool_to_01(left < right)),
        ">" => stack.push(bool_to_01(left > right)),
        "==" => stack.push(bool_to_01(left == right)),
        "!=" => stack.push(bool_to_01(left != right)),
        "/" => {
            if right == 0.0 {
                return Err(ExecError::Other("divided by 0".to_string()));
            }
            stack.push(ArithElem::Float(left / right));
        }
        "**" => {
            if right >= 0.0 {
                stack.push(ArithElem::Float(left.powf(right)));
            } else {
                return Err(ExecError::Other(error::exponent(&right.to_string())));
            }
        }
        _ => {
            return Err(ExecError::Other(
                "not supported operator for float numbers".to_string(),
            ))
        }
    }

    Ok(())
}

pub fn substitute(
    op: &str,
    name: &str,
    index: &str,
    cur: f64,
    right: f64,
    core: &mut ShellCore,
) -> Result<ArithElem, ExecError> {
    let new_value = match op {
        "+=" => cur + right,
        "-=" => cur - right,
        "*=" => cur * right,
        "/=" => match right == 0.0 {
            true => return Err(ArithError::DivZero(right.to_string()).into()),
            false => cur / right,
        },
        _ => return Err(ArithError::OperandExpected(op.to_string()).into()),
    };

    core.db
        .set_param2(name, index, &new_value.to_string(), None)?;
    Ok(ArithElem::Float(new_value))
}

pub fn parse(s: &str) -> Result<f64, ArithError> {
    let mut sw = s.to_string();
    let sign = variable::get_sign(&mut sw);

    match (sw.parse::<f64>(), sign.as_str()) {
        (Ok(f), "-") => Ok(-f),
        (Ok(f), _) => Ok(f),
        (Err(_), _) => Err(ArithError::InvalidNumber(s.to_string())),
    }
}
