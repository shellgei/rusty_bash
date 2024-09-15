//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{error_message, ShellCore};
use super::elem::Elem;
use super::{elem, float, int, rev_polish, trenary, word};

pub fn pop_operand(stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<Elem, String> {
    match stack.pop() {
        Some(Elem::Word(w, inc)) => word::to_operand(&w, 0, inc, core),
        Some(Elem::InParen(mut a)) => a.eval_elems(core, false),
        Some(elem) => Ok(elem),
        None       => Err("no operand".to_string()),
    }
}

fn bin_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    match op {
    "=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=" 
          => word::substitution(op, stack, core),
        _ => bin_calc_operation(op, stack, core),
    }
}

fn bin_calc_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
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
        (Elem::Float(fl), Elem::Float(fr)) => float::bin_calc(op, fl, fr, stack),
        (Elem::Float(fl), Elem::Integer(nr)) => float::bin_calc(op, fl, nr as f64, stack),
        (Elem::Integer(nl), Elem::Float(fr)) => float::bin_calc(op, nl as f64, fr, stack),
        (Elem::Integer(nl), Elem::Integer(nr)) => int::bin_calc(op, nl, nr, stack),
        _ => error_message::internal("invalid operand"),
    };
}

fn unary_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    let operand = match pop_operand(stack, core) {
        Ok(v)  => v,
        Err(e) => return Err(e),
    };

    match operand {
        Elem::Float(num)   => float::unary_calc(op, num, stack),
        Elem::Integer(num) => int::unary_calc(op, num ,stack),
        _ => error_message::internal("unknown operand"),
    }
}

pub fn calculate(elements: &Vec<Elem>, core: &mut ShellCore) -> Result<Elem, String> {
    if elements.len() == 0 {
        return Ok(Elem::Integer(0));
    }

    let rev_pol = match rev_polish::rearrange(elements) {
        Ok(ans) => ans,
        Err(e)  => return Err( error_message::syntax(&elem::to_string(&e)) ),
    };

    let mut stack = vec![];
    let mut skip_until = String::new();

    for e in rev_pol {
        if let Elem::BinaryOp(ref op) = e { //for short-circuit evaluation
            if op == &skip_until {
                skip_until = "".to_string();
                continue;
            }
        }else if skip_until != "" {
                continue;
        }

        /*
        dbg!("{:?}", &stack);
        dbg!("{:?}", &e);
        */

        let result = match e {
            Elem::Integer(_) | Elem::Float(_) | Elem::Word(_, _) | Elem::InParen(_) => {
                stack.push(e.clone());
                Ok(())
            },
            Elem::BinaryOp(ref op) => bin_operation(&op, &mut stack, core),
            Elem::UnaryOp(ref op)  => unary_operation(&op, &mut stack, core),
            Elem::Increment(n)     => inc(n, &mut stack, core),
            Elem::Ternary(left, right) => trenary::operation(&left, &right, &mut stack, core),
            Elem::Delimiter(d) => match check_skip(&d, &mut stack, core) {
                                    Ok(s) => {skip_until = s; Ok(())},
                                    Err(e) => Err(e),
                                  },
        };

        if let Err(err_msg) = result {
            return Err(err_msg);
        }
    }

    if stack.len() != 1 {
        return Err( format!("unknown syntax error_message (stack inconsistency)",) );
    }
    pop_operand(&mut stack, core)
}

fn check_skip(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<String, String> {
    let last = pop_operand(stack, core);
    let last_result = match &last {
        Err(e) => return Err(e.to_string()),
        Ok(Elem::Integer(0)) => 0,
        Ok(_) => 1,
    };

    stack.push(Elem::Integer(last_result));

    if last_result == 1 && op == "||" {
        return Ok("||".to_string());
    }
    if last_result == 0 && op == "&&" {
        return Ok("&&".to_string());
    }

    Ok("".to_string())
}

fn inc(inc: i64, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    match stack.pop() {
        Some(Elem::Word(w, inc_post)) => {
            match word::to_operand(&w, inc, inc_post, core) {
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
