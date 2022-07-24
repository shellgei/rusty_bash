//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder};
use crate::scanner::*;

fn _op_order(operator: &String) -> u8 {
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

fn get_integer(text: &mut Feeder) -> Option<String> {
    let pos = scanner_integer(&text, 0);

    if pos != 0 {
        Some(text.consume(pos))
    }else{
        None
    }
}

fn get_operator(text: &mut Feeder) -> Option<String> {
    if text.compare(0, "+") || text.compare(0, "-") || text.compare(0, "/") || text.compare(0, "%") {
        Some(text.consume(1))
    }else{
        None
    }
}

pub fn calculate(expression: String, core: &mut ShellCore) -> String {
    let tokens = tokenizer(expression, core);

    tokens.join(" ")
}

fn tokenizer(expression: String, _core: &mut ShellCore) -> Vec<String> {
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

