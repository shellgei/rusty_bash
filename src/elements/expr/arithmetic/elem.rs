//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod array_elem;
pub mod int;
pub mod float;
pub mod trenary;
pub mod word;

use super::ArithmeticExpr;
use super::Word;
use crate::ShellCore;
use crate::error::exec::ExecError;
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
    /* only for parse */
    Space(String),
    Symbol(String),
}

impl ArithElem {
    pub fn order(&self) -> u8 {
        match self {
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

    pub fn change_to_value(&mut self, add: i64, core: &mut ShellCore) -> Result<(), ExecError> {
        let tmp = match self {
            ArithElem::ArrayElem(name, ref mut sub, inc)
                => array_elem::to_operand(&name, sub, add, *inc, core)?,
            ArithElem::Word(w, inc) => word::to_operand(&w, add, *inc, core)?,
            ArithElem::InParen(ref mut a) => a.eval_elems(core, false)?,
            _ => return Ok(()),
        };

        *self = tmp;
        Ok(())
    }
}

