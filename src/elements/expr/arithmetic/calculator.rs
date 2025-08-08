//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::elem::ArithElem;
use super::elem::{float, int, ternary, variable};
use super::rev_polish;
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::utils::exit;
use crate::ShellCore;

pub fn pop_operand(
    stack: &mut Vec<ArithElem>,
    core: &mut ShellCore,
) -> Result<ArithElem, ExecError> {
    if let Some(mut e) = stack.pop() {
        e.change_to_value(0, core)?;
        return Ok(e);
    }

    Err(ExecError::Other("no operand 2".to_string()))
}

pub fn pop_operands(
    stack: &mut Vec<ArithElem>,
    core: &mut ShellCore,
) -> Result<(ArithElem, ArithElem), ExecError> {
    let right = stack.pop();
    let left = stack.pop();

    if let Some(mut left_v) = left {
        left_v.change_to_value(0, core)?;

        if let Some(mut right_v) = right {
            right_v.change_to_value(0, core)?;
            return Ok((left_v, right_v));
        }
    }
    Err(ExecError::Other("no operand 2".to_string()))
}

fn bin_operation(
    op: &str,
    stack: &mut Vec<ArithElem>,
    core: &mut ShellCore,
) -> Result<(), ExecError> {
    match op {
        "=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=" => {
            variable::substitution(op, stack, core)
        }
        "&&" | "||" => bin_calc_and_or(op, stack, core),
        _ => bin_calc_operation(op, stack, core),
    }
}

fn bin_calc_and_or(
    op: &str,
    stack: &mut Vec<ArithElem>,
    core: &mut ShellCore,
) -> Result<(), ExecError> {
    let mut right = match stack.pop() {
        Some(e) => e,
        None => return Err(ArithError::OperandExpected(op.to_string()).into()),
    };
    let mut left = match stack.pop() {
        Some(e) => e,
        None => return Err(ArithError::OperandExpected(op.to_string()).into()),
    };

    left.change_to_value(0, core)?;

    if let ArithElem::Integer(n) = left {
        if n == 0 && op == "&&" {
            stack.push(ArithElem::Integer(0));
            return Ok(());
        }

        if n != 0 && op == "||" {
            stack.push(ArithElem::Integer(1));
            return Ok(());
        }
    }

    right.change_to_value(0, core)?;

    if let ArithElem::Integer(n) = right {
        if n == 0 {
            stack.push(ArithElem::Integer(0));
        } else {
            stack.push(ArithElem::Integer(1));
        }
    }
    Ok(())
}

fn bin_calc_operation(
    op: &str,
    stack: &mut Vec<ArithElem>,
    core: &mut ShellCore,
) -> Result<(), ExecError> {
    let (left, right) = pop_operands(stack, core)?;

    if op == "," {
        stack.push(right);
        return Ok(());
    }

    let ans = match (left, right) {
        (ArithElem::Float(fl), ArithElem::Float(fr)) => float::bin_calc(op, fl, fr, stack)?,
        (ArithElem::Float(fl), ArithElem::Integer(nr)) => {
            float::bin_calc(op, fl, nr as f64, stack)?
        }
        (ArithElem::Integer(nl), ArithElem::Float(fr)) => {
            float::bin_calc(op, nl as f64, fr, stack)?
        }
        (ArithElem::Integer(nl), ArithElem::Integer(nr)) => int::bin_calc(op, nl, nr, stack)?,
        _ => exit::internal("invalid operand"),
    };

    Ok(ans)
}

fn unary_operation(
    op: &str,
    stack: &mut Vec<ArithElem>,
    core: &mut ShellCore,
) -> Result<(), ExecError> {
    let operand = match pop_operand(stack, core) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    match operand {
        ArithElem::Float(num) => float::unary_calc(op, num, stack),
        ArithElem::Integer(num) => int::unary_calc(op, num, stack),
        _ => exit::internal("unknown operand"),
    }
}

pub fn calculate(elements: &Vec<ArithElem>, core: &mut ShellCore) -> Result<ArithElem, ExecError> {
    if elements.is_empty() {
        return Ok(ArithElem::Integer(0));
    }

    let rev_pol = rev_polish::rearrange(elements)?;
    dry_run(&rev_pol)?;

    let mut stack = vec![];
    let mut escaped_unaries = vec![];

    for e in rev_pol {
        match e {
            ArithElem::BinaryOp(ref op) => bin_operation(&op, &mut stack, core)?,
            ArithElem::UnaryOp(ref op) => match stack.is_empty() {
                true => escaped_unaries.push(e),
                false => {
                    let mut ok = unary_operation(&op, &mut stack, core)?;
                    while !escaped_unaries.is_empty() {
                        match escaped_unaries.pop().unwrap() {
                            ArithElem::UnaryOp(ref op) => {
                                ok = unary_operation(&op, &mut stack, core)?;
                            }
                            _ => {}
                        }
                    }
                    ok
                }
            },
            ArithElem::Increment(n) => inc(n, &mut stack, core)?,
            ArithElem::Ternary(left, right) => ternary::operation(&left, &right, &mut stack, core)?,
            _ => stack.push(e.clone()),
        }
    }

    if stack.is_empty() {
        return Err(ArithError::OperandExpected(String::new()).into());
    }
    if stack.len() != 1 {
        return Err(ArithError::OperandExpected(stack.last().unwrap().to_string()).into());
    }
    pop_operand(&mut stack, core)
}

fn dry_run(rev_pol: &Vec<ArithElem>) -> Result<(), ArithError> {
    let mut stack = vec![];
    let mut last = None;

    for e in rev_pol {
        match e {
            ArithElem::BinaryOp(_) => {
                stack.pop();
                if stack.is_empty() {
                    return Err(ArithError::OperandExpected(e.to_string()));
                }
            }
            ArithElem::UnaryOp(_) | ArithElem::Increment(_) => last = Some(e),
            ArithElem::Ternary(_, _) => {
                if stack.is_empty() {
                    return Err(ArithError::OperandExpected(e.to_string()));
                }
            }
            _ => stack.push(e.clone()),
        }
    }

    if stack.is_empty() {
        match last {
            Some(e) => return Err(ArithError::OperandExpected(e.to_string())),
            None => return Err(ArithError::OperandExpected(String::new())),
        }
    }
    if stack.len() != 1 {
        return Err(ArithError::SyntaxError(stack.last().unwrap().to_string()));
    }
    Ok(())
}

fn inc(inc: i128, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), ExecError> {
    if let Some(mut op) = stack.pop() {
        op.change_to_value(inc, core)?;
        stack.push(op);
        Ok(())
    } else {
        Err(ArithError::OperandExpected("".to_string()).into())
    }
}
