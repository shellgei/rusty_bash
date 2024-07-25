//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use super::CalcElement;

fn op_order(operator: &str) -> u8 {
    let op: &str = &operator.clone();

    match op {
        "**" => 5,
        "*" | "/" | "%"            => 6, 
        "+" | "-"                  => 7, 
        "<<" | ">>"                => 8, 
        "<=" | ">=" | ">" | "<"    => 9, 
        "(" | ")"                  => 20, 
        _ => 255, 
    }
}

/*
fn get_integer(text: &mut Feeder) -> Option<(String,u8)> {
    let pos = text.scanner_integer();

    if pos != 0 {
        Some( (text.consume(pos),0) )
    }else{
        None
    }
}

fn get_operator(text: &mut Feeder) -> Option<(String,u8)> {
    if text.len() == 0 {
        return None;
    }

    if let Some(_) = "+-/%*".find(text.nth(0)) {
        let op = text.consume(1);
        Some( (op.clone(), op_order(&op)) )
    }else{
        None
    }
}*/

fn reduce(stack: &mut Vec<i32>, op: String ) {
    let op: &str = &op.clone();

    let right = stack.pop().unwrap();
    let left = stack.pop().unwrap();

    match op {
        "+" => stack.push(left+right),
        "-" => stack.push(left-right),
        "*" => stack.push(left*right),
        "/" => stack.push(left/right),
        _ => (), 
    }
}

fn to_op_str(calc_elem: Option<&CalcElement>) -> Option<&str> {
    match calc_elem {
        Some(CalcElement::Op(s)) => Some(&s),
        _ => None,
    }
}

fn rev_polish(elements: &Vec<CalcElement>) -> Vec<CalcElement> {
    let mut ans = vec![];
    let mut stack = vec![];

    for e in elements {
        match e {
            CalcElement::Num(n) => ans.push(CalcElement::Num(*n)),
            CalcElement::Op(s) => {
                loop {
                    match to_op_str(stack.last()) {
                        None | Some("(") => {
                            stack.push(CalcElement::Op(s.clone()));
                            break;
                        },
                        Some(")") => {
                            stack.pop();
                            loop {
                                match to_op_str(stack.last()) {
                                    None => {},
                                    Some("(") => {
                                        stack.pop();
                                        break;
                                    },
                                    Some(e) => ans.push(CalcElement::Op(e.to_string())),
                                }
                            }
                        },
                        Some(top_str) => {
                            if op_order(top_str) > op_order(s) {
                                stack.push(CalcElement::Op(s.clone()));
                                break;
                            }else{
                                ans.push(stack.pop().unwrap());
                            }
                        },
                    }
                }
            },
            _ => {},
        }
    }

    while stack.len() > 0 {
        ans.push(stack.pop().unwrap());
    }

    ans
}

pub fn calculate(elements: &Vec<CalcElement>) -> Option<String> {
    let rev_pol = rev_polish(&elements);
    dbg!("{:?}", &rev_pol);

    for e in &rev_pol {
        match e {
            CalcElement::Num(s) => return Some(s.to_string()),
            _ => return None,
        }
    }

    /*
    let tokens = tokenizer(expression, core);
    let mut num_stack: Vec<i32> = vec![];
    let mut wait_stack: Vec<(String,u8)> = vec![];

    for t in tokens {
        while wait_stack.len() != 0 {
            let wtop = wait_stack.pop().unwrap();

            if t.0 == ")" && wtop.0 == "(" {
            //    wait_stack.push(wtop);
                break;
            }

            if wtop.1 <= t.1 {
                if wtop.1 > 0 {
                    reduce(&mut num_stack, wtop.0.clone());
                }else{
                    num_stack.push(wtop.0.parse::<i32>().unwrap());
                }
            }else{
                wait_stack.push(wtop);
                break;
            }
        }

        if t.0 != ")" {
            wait_stack.push(t);
        }
    }

    while wait_stack.len() != 0 {
        let wtop = wait_stack.pop().unwrap();
        if wtop.1 > 0 {
            reduce(&mut num_stack, wtop.0.clone());
        }else{
            num_stack.push(wtop.0.parse::<i32>().unwrap());
        }
    }

    num_stack.pop().unwrap().to_string()
    */
    Some("0".to_string())
}

/*
fn tokenizer(expression: String, _core: &mut ShellCore) -> Vec<(String,u8)> {
    //let mut stack = vec![];
    let mut tokens = vec![];
    
    let mut text = Feeder::new_from(expression.clone());
    while text.len() != 0 {
        //get value
        if let Some(n) = get_integer(&mut text) {
            tokens.push(n);
        }else{
            break;
        }

        //get operator
        if let Some(op) = get_operator(&mut text) {
            tokens.push(op);
        }else{
            break;
        }
    }

    tokens
}*/

