//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod int;
pub mod float;
pub mod trenary;
pub mod variable;

use super::ArithmeticExpr;
use super::Word;
use crate::ShellCore;
use crate::error::exec::ExecError;
use crate::elements::subscript::Subscript;

#[derive(Debug, Clone)]
pub enum ArithElem {
    UnaryOp(String),
    BinaryOp(String),
    Integer(i128),
    Float(f64),
    Ternary(Box<Option<ArithmeticExpr>>, Box<Option<ArithmeticExpr>>),
    Variable(String, Option<Subscript>, i128), // name + subscript + post increment or decrement
    InParen(ArithmeticExpr),
    Increment(i128), //pre increment
    Delimiter(String), //delimiter dividing left and right of &&, ||, and ','
    /* only for parse */
    Space(String),
    Symbol(String),
    Word(Word, i128), // Word + post increment or decrement
    ArrayElem(String, Subscript, i128), // a[1]++
}

impl ArithElem {
    pub fn order(&self) -> u8 {
        match self {
            ArithElem::Increment(_) => 20,
            ArithElem::UnaryOp(s) => {
                match s.as_str() {
                    "-" | "+" => 19,
                    _         => 19,
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

    pub fn to_string(&self) -> String {
        match self {
            ArithElem::Space(s) => s.to_string(),
            ArithElem::Symbol(s) => s.to_string(),
            ArithElem::InParen(a) => a.text.to_string(),
            ArithElem::Integer(n) => n.to_string(),
            ArithElem::Float(f) => {
                let mut ans = f.to_string();
                if ! ans.contains('.') {
                    ans += ".0";
                }
                ans
            },
            ArithElem::Word(w, inc) => {
                match inc {
                    1  => w.text.clone() + "++",
                    -1 => w.text.clone() + "--",
                    _  => w.text.clone(),
                }
            },
            ArithElem::Variable(w, sub, inc) => {
                let mut ans = w.clone();
                if let Some(s) = sub {
                    ans += &s.text.clone();
                }
                match inc {
                    1  => ans += "++",
                    -1 => ans += "--",
                    _  => {},
                }
                ans
            },
            ArithElem::Ternary(left, right) => {
                let mut ans = "?".to_string();
                if let Some(e) = *left.clone() {
                    ans += &e.text.clone();
                }
                ans += ":";
                if let Some(e) = *right.clone() {
                    ans += &e.text.clone();
                }
                ans
            },
            ArithElem::UnaryOp(s) => s.clone(),
            ArithElem::BinaryOp(s) => s.clone(),
            ArithElem::Increment(1) => "++".to_string(),
            ArithElem::Increment(-1) => "--".to_string(),
            ArithElem::ArrayElem(name, subs, inc) => {
                let mut arr = name.clone() + &subs.text;
                match inc {
                    1  => arr += "++",
                    -1 => arr += "--",
                    _  => {},
                }
                arr
            },
            _ => "".to_string(),
        }
    }

    pub fn change_to_value(&mut self, add: i128, core: &mut ShellCore)
    -> Result<(), ExecError> {
        *self = match self {
            ArithElem::InParen(ref mut a) => a.eval_elems(core, false)?,
            ArithElem::Variable(w, s, inc) => {
                if add != 0 && *inc != 0 {
                    return Err(ExecError::OperandExpected(w.to_string()));
                }

                let index = match s {
                    Some(sub) => sub.eval(core, &w)?,
                    None => "".to_string(),
                };

                match add {
                    0 => variable::set_and_to_value(&w, &index, core, *inc, false)?,
                    _ => variable::set_and_to_value(&w, &index, core, add, true)?,
                }
            },
            _ => return Ok(()),
        };
        Ok(())
    }
}

