//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use crate::scanner::*;

fn op_order(operator: &String) -> u8 {
    let op: &str = &operator.clone();

    match op {
        "**" => 5,
        "*" | "/" | "%"            => 6, 
        "+" | "-"                  => 7, 
        "<<" | ">>"                => 8, 
        "<=" | ">=" | ">" | "<"    => 9, 
        _ => 255, 
    }
}

fn get_integer(text: &mut Feeder) -> Option<(String,u8)> {
    let pos = scanner_integer(&text, 0);

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
}

fn reduce(stack: &mut Vec<(String,u8)>, op: String ) {
}

pub fn calculate(expression: String, core: &mut ShellCore) -> String {
    let tokens = tokenizer(expression, core);
    let mut stack = vec!();
//    let mut num_stack = vec!();
    let mut wait_stack: Vec<(String,u8)> = vec!();

    eprintln!("TOKENS: {:?}", tokens);

    for t in tokens {
        eprintln!("STACK: {:?}", stack);
        eprintln!("WAIT STACK: {:?}", wait_stack);
        while wait_stack.len() != 0 {
            let wtop = wait_stack.pop().unwrap();
            if wtop.1 < t.1 {
                if wtop.1 > 0 {
                    //reduce(&mut stack, wtop.0.clone());
                    stack.push(wtop);
                }

            }else{
                wait_stack.push(wtop);
                break;
            }
        }
        wait_stack.push(t);
    }

    while wait_stack.len() != 0 {
        let wtop = wait_stack.pop().unwrap();
        stack.push(wtop);
    }

    stack.iter().map(|t| t.0.clone()).collect::<Vec<String>>().join(" ")
}

fn tokenizer(expression: String, _core: &mut ShellCore) -> Vec<(String,u8)> {
    //let mut stack = vec!();
    let mut tokens = vec!();
    
    let mut text = Feeder::new_with(expression.clone());
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
}

