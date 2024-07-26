//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::CalcElement;

fn op_order(op: &CalcElement) -> u8 {
    match op {
        CalcElement::UnaryOp(_) => 8,
        CalcElement::BinaryOp(s) => {
            match s.as_str() {
                "*" | "/" | "%" => 5, 
                "+" | "-"       => 4, 
                _ => 0,
            }
        },
        _ => 0, 
    }
}

fn rev_polish(elements: &Vec<CalcElement>) -> Vec<CalcElement> {
    let mut ans = vec![];
    let mut stack = vec![];

    for e in elements {
        match e {
            CalcElement::LeftParen => {
                stack.push(e.clone());
                continue;
            },
            CalcElement::RightParen => {
                loop {
                    match stack.last() {
                        None => {},
                        Some(CalcElement::LeftParen) => {
                            stack.pop();
                            break;
                        },
                        Some(_) => ans.push(stack.pop().unwrap()),
                    }
                }
                continue;
            },
            CalcElement::Num(n) => ans.push(CalcElement::Num(*n)),
            CalcElement::UnaryOp(_) | CalcElement::BinaryOp(_) => {
                loop {
                    match stack.last() {
                        None | Some(CalcElement::LeftParen) => {
                            stack.push(e.clone());
                            break;
                        },
                        Some(_) => {
                            let last = stack.last().unwrap();
                            if op_order(last) <= op_order(e) {
                                stack.push(e.clone());
                                break;
                            }
                            ans.push(stack.pop().unwrap());
                        },
                    }
                }
            },
        }
    }

    while stack.len() > 0 {
        ans.push(stack.pop().unwrap());
    }

    ans
}

fn operation_minus(stack: &mut Vec<CalcElement>) {
    if stack.len() < 2 {
        panic!("SUSH INTERNAL ERROR: wrong operation");
    }

    let right = match stack.pop() {
        Some(CalcElement::Num(s)) => s,
        _ => panic!("SUSH INTERNAL ERROR: wrong operation"),
    };

    let left = match stack.pop() {
        Some(CalcElement::Num(s)) => s,
        _ => panic!("SUSH INTERNAL ERROR: wrong operation"),
    };

    stack.push( CalcElement::Num(left - right) );
}

fn operation_plus(stack: &mut Vec<CalcElement>) {
    if stack.len() < 2 {
        panic!("SUSH INTERNAL ERROR: wrong operation");
    }

    let right = match stack.pop() {
        Some(CalcElement::Num(s)) => s,
        _ => panic!("SUSH INTERNAL ERROR: wrong operation"),
    };

    let left = match stack.pop() {
        Some(CalcElement::Num(s)) => s,
        _ => panic!("SUSH INTERNAL ERROR: wrong operation"),
    };

    stack.push( CalcElement::Num(right + left) );
}

fn bin_operation(op: &str, stack: &mut Vec<CalcElement>) {
    match op {
        "+" => operation_plus(stack),
        "-" => operation_minus(stack),
        _ => {},
    }
}

fn unary_operation(op: &str, stack: &mut Vec<CalcElement>) {
    let num = match stack.pop() {
        Some(CalcElement::Num(s)) => s,
        _ => panic!("SUSH INTERNAL ERROR: wrong operation"),
    };

    match op {
        "+" => stack.push( CalcElement::Num(num) ),
        "-" => stack.push( CalcElement::Num(-num) ),
        _ => {},
    }
}


pub fn calculate(elements: &Vec<CalcElement>) -> Option<CalcElement> {
    let rev_pol = rev_polish(&elements);
    let mut stack = vec![];

    for e in rev_pol {
        match e {
            CalcElement::Num(_) => stack.push(e),
            CalcElement::BinaryOp(op) => bin_operation(&op, &mut stack),
            CalcElement::UnaryOp(op) => unary_operation(&op, &mut stack),
            _ => {}, 
        }
    }

    if stack.len() != 1 {
        panic!("SUSH INTERNAL ERROR: wrong operation");
    }

    stack.pop()
}
