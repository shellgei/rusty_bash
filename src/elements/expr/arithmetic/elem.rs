//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod float;
pub mod int;
pub mod ternary;
pub mod variable;

use super::ArithmeticExpr;
use super::Word;
use crate::elements::substitution::subscript::Subscript;
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::{utils, ShellCore};
use std::fmt;

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
    //    Delimiter(String), //delimiter dividing left and right of &&, ||, and ','
    /* only for parse */
    Space(String),
    Symbol(String),
    Word(Word, i128),                   // Word + post increment or decrement
    ArrayElem(String, Subscript, i128), // a[1]++
}

impl ArithElem {
    pub fn order(&self) -> u8 {
        match self {
            ArithElem::Increment(_) => 20,
            ArithElem::UnaryOp(s) => match s.as_str() {
                "-" | "+" => 19,
                _ => 19,
            },
            ArithElem::BinaryOp(s) => {
                match s.as_str() {
                    "**" => 17,
                    "*" | "/" | "%" => 16,
                    "+" | "-" => 15,
                    "<<" | ">>" => 14,
                    "<=" | ">=" | ">" | "<" => 13,
                    "==" | "!=" => 12,
                    "&" => 11,
                    "^" => 10,
                    "|" => 9,
                    "&&" => 8,
                    "||" => 7,
                    "," => 0,
                    _ => 2, //substitution
                }
            }
            ArithElem::Ternary(_, _) => 3,
            _ => 1,
        }
    }
}

impl fmt::Display for ArithElem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArithElem::Space(s) | ArithElem::Symbol(s) => write!(f, "{s}"),
            ArithElem::InParen(a) => write!(f, "{}", a.text),
            ArithElem::Integer(n) => write!(f, "{n}"),
            ArithElem::Float(val) => {
                let mut s = val.to_string();
                if !s.contains('.') {
                    s.push_str(".0");
                }
                write!(f, "{s}")
            }
            ArithElem::Word(w, inc) => match inc {
                1 => write!(f, "{}++", w.text),
                -1 => write!(f, "{}--", w.text),
                _ => write!(f, "{}", w.text),
            },
            ArithElem::Variable(w, sub, inc) => {
                if let Some(s) = sub {
                    match inc {
                        1 => write!(f, "{}{}++", w, s.text),
                        -1 => write!(f, "{}{}--", w, s.text),
                        _ => write!(f, "{}{}", w, s.text),
                    }
                } else {
                    match inc {
                        1 => write!(f, "{w}++"),
                        -1 => write!(f, "{w}--"),
                        _ => write!(f, "{w}"),
                    }
                }
            }
            ArithElem::Ternary(left, right) => {
                let mut s = String::from("?");
                if let Some(e) = *left.clone() {
                    s += &e.text;
                }
                s.push(':');
                if let Some(e) = *right.clone() {
                    s += &e.text;
                }
                write!(f, "{s}")
            }
            ArithElem::UnaryOp(s) | ArithElem::BinaryOp(s) => write!(f, "{s}"),
            ArithElem::Increment(n) => {
                if *n > 0 {
                    write!(f, "++")
                } else if *n < 0 {
                    write!(f, "--")
                } else {
                    write!(f, "")
                }
            }
            ArithElem::ArrayElem(name, subs, inc) => match inc {
                1 => write!(f, "{}{}++", name, subs.text),
                -1 => write!(f, "{}{}--", name, subs.text),
                _ => write!(f, "{}{}", name, subs.text),
            },
        }
    }
}

impl ArithElem {
    pub fn change_to_value(&mut self, add: i128, core: &mut ShellCore) -> Result<(), ExecError> {
        *self = match self {
            ArithElem::InParen(a) => a.eval_elems(core, false)?,
            ArithElem::Variable(name, s, inc) => {
                if add != 0 && *inc != 0 || !utils::is_name(name, core) {
                    return Err(ArithError::OperandExpected(name.to_string()).into());
                }

                let index = match s {
                    Some(sub) => sub.eval(core, name)?,
                    None => "".to_string(),
                };

                match add {
                    0 => variable::set_and_to_value(name, &index, core, *inc, false)?,
                    _ => variable::set_and_to_value(name, &index, core, add, true)?,
                }
            }
            _ => return Ok(()),
        };
        Ok(())
    }

    pub fn is_operand(&self) -> bool {
        matches!(
            self,
            ArithElem::Float(_)
                | ArithElem::Integer(_)
                | ArithElem::ArrayElem(_, _, _)
                | ArithElem::Word(_, _)
                | ArithElem::Variable(_, _, _)
                | ArithElem::InParen(_)
        )
    }
}
