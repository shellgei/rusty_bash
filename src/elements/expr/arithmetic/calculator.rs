//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::error::exec::ExecError;
use crate::utils::exit;
use super::elem::ArithElem;
use super::{float, int, rev_polish, trenary, word, array_elem};

pub fn pop_operand(stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<ArithElem, ExecError> {
    match stack.pop() {
        Some(ArithElem::ArrayElem(name, mut sub, inc))
            => array_elem::to_operand(&name, &mut sub, 0, inc, core),
        Some(ArithElem::Word(w, inc)) => word::to_operand(&w, 0, inc, core),
        Some(ArithElem::InParen(mut a)) => a.eval_elems(core, false),
        Some(elem) => Ok(elem),
        None       => Err(ExecError::Other("no operand 2".to_string())),
    }
}

fn bin_operation(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), ExecError> {
   match op {
    "=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=" 
          => word::substitution(op, stack, core),
        _ => bin_calc_operation(op, stack, core),
    }
}

fn bin_calc_operation(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore)
    -> Result<(), ExecError> {
    let right = pop_operand(stack, core)?;
    let left = pop_operand(stack, core)?;

    if op == "," {
        stack.push(right);
        return Ok(());
    }

    return match (left, right) {
        (ArithElem::Float(fl), ArithElem::Float(fr)) => float::bin_calc(op, fl, fr, stack),
        (ArithElem::Float(fl), ArithElem::Integer(nr)) => float::bin_calc(op, fl, nr as f64, stack),
        (ArithElem::Integer(nl), ArithElem::Float(fr)) => float::bin_calc(op, nl as f64, fr, stack),
        (ArithElem::Integer(nl), ArithElem::Integer(nr)) => int::bin_calc(op, nl, nr, stack),
        _ => exit::internal("invalid operand"),
    };
}

fn unary_operation(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), ExecError> {
    let operand = match pop_operand(stack, core) {
        Ok(v)  => v,
        Err(e) => return Err(e),
    };

    match operand {
        ArithElem::Float(num)   => float::unary_calc(op, num, stack),
        ArithElem::Integer(num) => int::unary_calc(op, num ,stack),
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
    let mut skip_until = String::new();

    for e in rev_pol {
        if let ArithElem::BinaryOp(ref op) = e { //for short-circuit evaluation
            if op == &skip_until {
                skip_until = "".to_string();
                continue;
            }
        }

        if skip_until != "" {
                continue;
        }

        match e {
            ArithElem::BinaryOp(ref op) => bin_operation(&op, &mut stack, core)?,
            ArithElem::UnaryOp(ref op)  => unary_operation(&op, &mut stack, core)?,
            ArithElem::Increment(n)     => inc(n, &mut stack, core)?,
            ArithElem::Ternary(left, right) => trenary::operation(&left, &right, &mut stack, core)?,
            ArithElem::Delimiter(d) => skip_until = check_skip(&d, &mut stack, core)?,
            _ => stack.push(e.clone()),
        }
    }

    if stack.is_empty() {
        return Err( ExecError::OperandExpected(String::new()));
    }
    if stack.len() != 1 {
        return Err( ExecError::OperandExpected(stack.last().unwrap().to_string()));
    }
    pop_operand(&mut stack, core)
}

fn dry_run(rev_pol: &Vec<ArithElem>) -> Result<(), ExecError> {
    let mut stack = vec![];

    for e in rev_pol {
        match e {
            ArithElem::BinaryOp(_) => {
                stack.pop();
                if stack.is_empty() {
                    return Err( ExecError::OperandExpected(e.to_string()));
                }
            },
            ArithElem::UnaryOp(_) 
            | ArithElem::Increment(_)
            | ArithElem::Ternary(_, _) => {
                if stack.is_empty() {
                    return Err( ExecError::OperandExpected(e.to_string()));
                }
            },
            ArithElem::Delimiter(_) => {},
            _ => { stack.push(e.clone()) },
        }
    }

    if stack.is_empty() {
        return Err( ExecError::OperandExpected(String::new()));
    }
    if stack.len() != 1 {
        return Err( ExecError::OperandExpected(stack.last().unwrap().to_string()));
    }
    Ok(())
}

fn check_skip(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<String, ExecError> {
    let last_result = match pop_operand(stack, core)? {
        ArithElem::Integer(0) => 0,
        _ => 1,
    };

    stack.push(ArithElem::Integer(last_result));

    if last_result == 1 && op == "||" {
        return Ok("||".to_string());
    }
    if last_result == 0 && op == "&&" {
        return Ok("&&".to_string());
    }

    Ok("".to_string())
}

fn inc(inc: i64, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), ExecError> {
    let op = match stack.pop() {
        Some(ArithElem::Word(w, inc_post)) => word::to_operand(&w, inc, inc_post, core)?,
        Some(ArithElem::ArrayElem(name, mut sub, inc_post)) => {
            array_elem::to_operand(&name, &mut sub, inc, inc_post, core)?
        },
        Some(e) => return Err(ExecError::OperandExpected(e.to_string())),
        None => return Err(ExecError::OperandExpected("".to_string())),
    };
    stack.push(op);
    Ok(())
}
