//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use super::{ArithmeticExpr, Elem};
use super::syntax_error_msg;
use super::word_manip;
use super::float_manip;
use super::int_manip;

pub fn exponent_error_msg(s: &str) -> String {
    format!("exponent less than 0 (error token is \"{}\")", s)
}

fn assignment_error_msg(right: &str) -> String {
    format!("attempted assignment to non-variable (error token is \"{}\")", right)
}

fn op_order(op: &Elem) -> u8 {
    match op {
        Elem::Increment(_) => 14,
        Elem::UnaryOp(s) => {
            match s.as_str() {
                "-" | "+" => 14,
                _         => 13,
            }
        },
        Elem::BinaryOp(s) => {
            match s.as_str() {
                "**"            => 12, 
                "*" | "/" | "%" => 11, 
                "+" | "-"       => 10, 
                "<<" | ">>"     => 9, 
                "<=" | ">=" | ">" | "<" => 8, 
                "==" | "!="     => 7, 
                "&"             => 6, 
                "^"             => 5, 
                "|"             => 4, 
                _               => 2,
                //_ => panic!("SUSH INTERNAL ERROR: unknown binary operator"),
            }
        },
        Elem::ConditionalOp(_, _) => 1,
        _ => 0, 
    }
}

fn to_string(op: &Elem) -> String {
    match op {
        Elem::Integer(n) => n.to_string(),
        Elem::Float(f) => f.to_string(),
        Elem::Word(w, inc) => {
            match inc {
                1  => w.text.clone() + "++",
                -1 => w.text.clone() + "--",
                _  => w.text.clone(),
            }
        },
        Elem::UnaryOp(s) => s.clone(),
        Elem::BinaryOp(s) => s.clone(),
        Elem::LeftParen => "(".to_string(),
        Elem::RightParen => ")".to_string(),
        Elem::Increment(1) => "++".to_string(),
        Elem::Increment(-1) => "--".to_string(),
        _ => "".to_string(),
    }
}

fn rev_polish(elements: &[Elem]) -> Result<Vec<Elem>, Elem> {
    let mut ans = vec![];
    let mut stack = vec![];
    let mut last = None;

    for e in elements {
        let ok = match e {
            Elem::Float(_) | Elem::Integer(_) | Elem::Word(_, _)
                             => {ans.push(e.clone()); true},
            Elem::LeftParen  => {stack.push(e.clone()); true},
            Elem::RightParen => rev_polish_paren(&mut stack, &mut ans),
            op               => rev_polish_op(&op, &mut stack, &mut ans),
        };

        if !ok {
            return Err(e.clone());
        }

        match (last, e) {
            ( Some(Elem::LeftParen), Elem::RightParen ) => return Err(e.clone()),
            _ => {},
        }

        last = Some(e.clone());
    }

    while stack.len() > 0 {
        ans.push(stack.pop().unwrap());
    }

    Ok(ans)
}

fn rev_polish_paren(stack: &mut Vec<Elem>, ans: &mut Vec<Elem>) -> bool {
    loop {
        match stack.last() {
            None => return false, 
            Some(Elem::LeftParen) => {
                stack.pop();
                return true;
            },
            Some(_) => ans.push(stack.pop().unwrap()),
        }
    }
}

fn rev_polish_op(elem: &Elem,
                 stack: &mut Vec<Elem>, ans: &mut Vec<Elem>) -> bool {
    loop {
        match stack.last() {
            None | Some(Elem::LeftParen) => {
                stack.push(elem.clone());
                break;
            },
            Some(_) => {
                let last = stack.last().unwrap();
                if op_order(last) <= op_order(elem) {
                    stack.push(elem.clone());
                    break;
                }
                ans.push(stack.pop().unwrap());
            },
        }
    }

    true
}

fn pop_operands(num: usize, stack: &mut Vec<Elem>,
                core: &mut ShellCore) -> Result<Vec<Elem>, String> {
    let mut ans = vec![];

    for _ in 0..num {
        let n = match stack.pop() {
            Some(Elem::Integer(s)) => Elem::Integer(s),
            Some(Elem::Float(f)) => Elem::Float(f),
            Some(Elem::Word(w, inc)) => {
                match word_manip::to_operand(&w, 0, inc, core) {
                    Ok(Elem::Integer(n)) => Elem::Integer(n),
                    Ok(Elem::Float(f))   => Elem::Float(f),
                    Err(e)               => return Err(e),
                    _ => panic!("SUSH INTERNAL ERROR: word_to_operand"),
                }
            },
            _ => return Ok(vec![]),
        };
        ans.push(n);
    }
    Ok(ans)
}

fn bin_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    match op {
    "=" | "*=" | "/=" | "%=" | "+=" | "-=" | "<<=" | ">>=" | "&=" | "^=" | "|=" 
          => substitution(op, stack, core),
        _ => bin_calc_operation(op, stack, core),
    }


}

fn substitution(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore)-> Result<(), String> {
    let right = match pop_operands(1, stack, core) {
        Ok(v) => {
            match v.len() == 1 {
                true  => v[0].clone(),
                false => return Err( syntax_error_msg(op) ),
            }
        },
        Err(e)  => return Err(e),
    };

    let left = match stack.pop() {
        Some(Elem::Word(w, 0)) => w,
        Some(Elem::Word(_, _)) => return Err( assignment_error_msg(op) ),
        _ => return Err( assignment_error_msg(op) ),
    };

    match word_manip::substitute(op, &left, &right, core) {
        Ok(elem) => stack.push(elem),
        Err(msg) => return Err(msg),
    }
    Ok(())
}

fn bin_calc_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    let (left, right) = match pop_operands(2, stack, core) {
        Ok(v) => {
            match v.len() == 2 {
                true  => (v[1].clone(), v[0].clone()), 
                false => return Err( syntax_error_msg(op) ),
            }
        },
        Err(e)  => return Err(e),
    };

    return match (left, right) {
        (Elem::Float(fl), Elem::Float(fr)) => float_manip::bin_calc(op, fl, fr, stack),
        (Elem::Float(fl), Elem::Integer(nr)) => float_manip::bin_calc(op, fl, nr as f64, stack),
        (Elem::Integer(nl), Elem::Float(fr)) => float_manip::bin_calc(op, nl as f64, fr, stack),
        (Elem::Integer(nl), Elem::Integer(nr)) => int_manip::bin_calc(op, nl, nr, stack),
        _ => panic!("SUSH INTERNAL ERROR: invalid operand"),
    };
}

fn unary_operation(op: &str, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    let operand = match pop_operands(1, stack, core) {
        Ok(v) => {
            match v.len() == 1 {
                true  => v[0].clone(),
                false => return Err( syntax_error_msg(op) ),
            }
        },
        Err(e)  => return Err(e),
    };

    match operand {
        Elem::Float(num) => match op {
            "+"  => stack.push( Elem::Float(num) ),
            "-"  => stack.push( Elem::Float(-num) ),
            _ => return Err("not supported operator for float number".to_string()),
        }
        Elem::Integer(num) => match op {
            "+"  => stack.push( Elem::Integer(num) ),
            "-"  => stack.push( Elem::Integer(-num) ),
            "!"  => stack.push( Elem::Integer(if num == 0 { 1 } else { 0 }) ),
            "~"  => stack.push( Elem::Integer( !num ) ),
            _ => panic!("SUSH INTERNAL ERROR: unknown unary operator"),
        }
        _ => panic!("SUSH INTERNAL ERROR: unknown operand"),
    }

    Ok(())
}

fn cond_operation(left: &Option<ArithmeticExpr>, right: &Option<ArithmeticExpr>,
    stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    let num = match pop_operands(1, stack, core) {
        Ok(v) => {
            match v.len() == 1 {
                true  => v[0].clone(),
                false => return Err( syntax_error_msg("?") ),
            }
        },
        Err(e)  => return Err(e),
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

pub fn calculate(elements: &Vec<Elem>, core: &mut ShellCore) -> Result<Elem, String> {
    let mut comma_pos = vec![];
    for (i, e) in elements.iter().enumerate() {
        match e {
            Elem::BinaryOp(c) => {
                if c == "," {
                    comma_pos.push(i);
                }
            },
            _ => {},
        }
    }

    let mut left = 0;
    for i in 0..comma_pos.len() {
        let right = comma_pos[i];
        if let Err(e) = calculate_sub(&elements[left..right], core) {
            return Err(e);
        }
        left = right + 1;
    }

    calculate_sub(&elements[left..], core)
}

fn calculate_sub(elements: &[Elem], core: &mut ShellCore) -> Result<Elem, String> {
    if elements.len() == 0 {
        return Ok(Elem::Integer(0));
    }

    let rev_pol = match rev_polish(elements) {
        Ok(ans) => ans,
        Err(e)  => return Err( syntax_error_msg(&to_string(&e)) ),
    };

    let mut stack = vec![];

    for e in rev_pol {
        let result = match e {
            Elem::Integer(_) | Elem::Float(_) | Elem::Word(_, _) => {
                stack.push(e.clone());
                Ok(())
            },
            Elem::BinaryOp(ref op) => bin_operation(&op, &mut stack, core),
            Elem::UnaryOp(ref op)  => unary_operation(&op, &mut stack, core),
            Elem::Increment(n)     => inc(n, &mut stack, core),
            Elem::ConditionalOp(left, right) => cond_operation(&left, &right, &mut stack, core),
            _ => Err( syntax_error_msg(&to_string(&e)) ),
        };

        if let Err(err_msg) = result {
            return Err(err_msg);
        }
    }

    if stack.len() != 1 {
        return Err( format!("unknown syntax error (stack inconsistency)",) );
    }

    match stack.pop() {
        Some(Elem::Integer(n)) => Ok(Elem::Integer(n)),
        Some(Elem::Float(f)) => Ok(Elem::Float(f)),
        Some(Elem::Word(w, inc)) => {
            match word_manip::to_operand(&w, 0, inc, core) {
                Ok(Elem::Integer(n)) => Ok(Elem::Integer(n)),
                Ok(Elem::Float(f)) => Ok(Elem::Float(f)),
                Err(e) => Err(e),
                _      => Err("unknown word parse error".to_string()),
            }
        },
        _ => Err( format!("unknown syntax error",) ),
    }
}

fn inc(inc: i64, stack: &mut Vec<Elem>, core: &mut ShellCore) -> Result<(), String> {
    match stack.pop() {
        Some(Elem::Word(w, inc_post)) => {
            match word_manip::to_operand(&w, inc, inc_post, core) {
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
