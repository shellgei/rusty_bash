//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::{error, exit};
use super::elem::ArithElem;
use super::{float, int, rev_polish, trenary, word, array_elem};

pub fn pop_operand(stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<ArithElem, String> {
    match stack.pop() {
        Some(ArithElem::ArrayElem(name, mut sub, inc))
            => array_elem::to_operand(&name, &mut sub, 0, inc, core),
        Some(ArithElem::Word(w, inc)) => word::to_operand(&w, 0, inc, core),
        Some(ArithElem::InParen(mut a)) => a.eval_elems(core, false),
        Some(elem) => Ok(elem),
        None       => Err("no operand 2".to_string()),
    }
}

fn bin_operation(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), String> {
    match op {
    "=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=" 
          => word::substitution(op, stack, core),
        _ => bin_calc_operation(op, stack, core),
    }
}

fn bin_calc_operation(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), String> {
    let right = match pop_operand(stack, core) {
        Ok(v)  => v,
        Err(e) => return Err(e),
    };

    let left = match pop_operand(stack, core) {
        Ok(v)  => v,
        Err(e) => return Err(e),
    };

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

fn unary_operation(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), String> {
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

pub fn calculate(elements: &Vec<ArithElem>, core: &mut ShellCore) -> Result<ArithElem, String> {
    if elements.is_empty() {
        return Ok(ArithElem::Integer(0));
    }

    let rev_pol = match rev_polish::rearrange(elements) {
        Ok(ans) => ans,
        Err(e)  => return Err( error::syntax(&e.to_string()) ),
    };

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

        let result = match e {
            ArithElem::Integer(_) 
            | ArithElem::Float(_) 
            | ArithElem::ArrayElem(_, _, _) 
            | ArithElem::Word(_, _) 
            | ArithElem::InParen(_) => {
                stack.push(e.clone());
                Ok(())
            },
            ArithElem::BinaryOp(ref op) => bin_operation(&op, &mut stack, core),
            ArithElem::UnaryOp(ref op)  => unary_operation(&op, &mut stack, core),
            ArithElem::Increment(n)     => inc(n, &mut stack, core),
            ArithElem::Ternary(left, right) => trenary::operation(&left, &right, &mut stack, core),
            ArithElem::Delimiter(d) => match check_skip(&d, &mut stack, core) {
                                    Ok(s) => {skip_until = s; Ok(())},
                                    Err(e) => Err(e),
                                  },
        };

        if let Err(err_msg) = result {
            return Err(err_msg);
        }
    }

    dbg!("{:?}", &stack);
    if stack.len() != 1 {
        return Err( format!("unknown syntax error (stack inconsistency)",) );
    }
    pop_operand(&mut stack, core)
}

fn check_skip(op: &str, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<String, String> {
    let last = pop_operand(stack, core);
    let last_result = match &last {
        Err(e) => return Err(e.to_string()),
        Ok(ArithElem::Integer(0)) => 0,
        Ok(_) => 1,
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

fn inc(inc: i64, stack: &mut Vec<ArithElem>, core: &mut ShellCore) -> Result<(), String> {
    match stack.pop() {
        Some(ArithElem::Word(w, inc_post)) => {
            match word::to_operand(&w, inc, inc_post, core) {
                Ok(op) => {
                    stack.push(op);
                    Ok(())
                },
                Err(e) => Err(e),
            }
        },
        Some(ArithElem::ArrayElem(name, mut sub, inc_post)) => {
            match array_elem::to_operand(&name, &mut sub, inc, inc_post, core) {
                Ok(op) => {
                    stack.push(op);
                    Ok(())
                },
                Err(e) => Err(e),
            }
        },
        _ => Err("invalid increment".to_string()),
    }
}
