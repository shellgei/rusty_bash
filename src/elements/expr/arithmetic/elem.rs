//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::ArithmeticExpr;
use super::Word;
use crate::elements::subscript::Subscript;

#[derive(Debug, Clone)]
pub enum ArithElem {
    UnaryOp(String),
    BinaryOp(String),
    Integer(i64),
    Float(f64),
    Ternary(Box<Option<ArithmeticExpr>>, Box<Option<ArithmeticExpr>>),
    ArrayElem(String, Subscript, i64), // a[1]++
    Word(Word, i64), // Word + post increment or decrement
    InParen(ArithmeticExpr),
    Increment(i64), //pre increment
    Delimiter(String), //delimiter dividing left and right of &&, ||, and ','
}

pub fn op_order(op: &ArithElem) -> u8 {
    match op {
        ArithElem::Increment(_) => 20,
        ArithElem::UnaryOp(s) => {
            match s.as_str() {
                "-" | "+" => 19,
                _         => 18,
            }
        },
        ArithElem::BinaryOp(s) => {
            match s.as_str() {
                "**"            => 17, 
                "*" | "/" | "%" => 16, 
                "+" | "-"       => 15, 
                "<<" | ">>"     => 14, 
                "<=" | ">=" | ">" | "<" => 13, 
                "==" | "!="     => 12, 
                "&"             => 11, 
                "^"             => 10, 
                "|"             => 9, 
                "&&"             => 8, 
                "||"             => 7, 
                ","             => 0, 
                _               => 2, //substitution
            }
        },
        ArithElem::Ternary(_, _) => 3,
        _ => 1, 
    }
}

pub fn to_string(op: &ArithElem) -> String {
    match op {
        ArithElem::InParen(a) => a.text.to_string(),
        ArithElem::Integer(n) => n.to_string(),
        ArithElem::Float(f) => f.to_string(),
        ArithElem::Word(w, inc) => {
            match inc {
                1  => w.text.clone() + "++",
                -1 => w.text.clone() + "--",
                _  => w.text.clone(),
            }
        },
        ArithElem::UnaryOp(s) => s.clone(),
        ArithElem::BinaryOp(s) => s.clone(),
        ArithElem::Increment(1) => "++".to_string(),
        ArithElem::Increment(-1) => "--".to_string(),
        _ => "".to_string(),
    }
}
