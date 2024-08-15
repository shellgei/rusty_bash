//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod calculator;
mod parser;
mod word;
mod int;
mod float;

use crate::ShellCore;
use self::calculator::calculate;
use super::word::Word;

#[derive(Debug, Clone)]
enum Elem {
    UnaryOp(String),
    BinaryOp(String),
    Integer(i64),
    Float(f64),
    ConditionalOp(Box<Option<ArithmeticExpr>>, Box<Option<ArithmeticExpr>>),
    Word(Word, i64), // Word + post increment or decrement
    LeftParen,
    RightParen,
    Increment(i64), //pre increment
}

#[derive(Debug, Clone)]
pub struct ArithmeticExpr {
    pub text: String,
    elements: Vec<Elem>,
    paren_stack: Vec<char>,
}

fn syntax_error_msg(token: &str) -> String {
    format!("{0}: syntax error: operand expected (error token is \"{0}\")", token)
}

impl ArithmeticExpr {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<String> {
        let es = match self.decompose_increments() {
            Ok(data)     => data, 
            Err(err_msg) => {
                eprintln!("sush: {}", err_msg);
                return None;
            },
        };

        match calculate(&es, core) {
            Ok(Elem::Integer(n))  => Some(n.to_string()),
            Ok(Elem::Float(f))  => Some(f.to_string()),
            Err(msg) => {
                eprintln!("sush: {}: {}", &self.text, msg);
                None
            },
            _ => panic!("SUSH INTERNAL ERROR: invalid calculation result"),
        }
    }

    fn eval_in_cond(&mut self, core: &mut ShellCore) -> Result<Elem, String> {
        let es = match self.decompose_increments() {
            Ok(data)     => data, 
            Err(err_msg) => return Err(err_msg),
        };

        match calculate(&es, core) {
            Ok(ans)      => Ok(ans),
            Err(err_msg) => return Err(err_msg),
        }
    }

    fn preinc_to_unarys(&mut self, ans: &mut Vec<Elem>, pos: usize, inc: i64) -> i64 {
        let pm = match inc {
            1  => "+",
            -1 => "-",
            _ => return 0,
        }.to_string();
    
        match (&ans.last(), &self.elements.iter().nth(pos+1)) {
            (_, None) 
            | (_, Some(&Elem::Word(_, _))) => return inc,
            (Some(&Elem::Integer(_)), _)
            | (Some(&Elem::Float(_)), _)   => ans.push(Elem::BinaryOp(pm.clone())),
            _                              => ans.push(Elem::UnaryOp(pm.clone())),
        }
        ans.push(Elem::UnaryOp(pm));
        0
    }

    fn decompose_increments(&mut self) -> Result<Vec<Elem>, String> {
        let mut ans = vec![];
        let mut pre_increment = 0;

        let len = self.elements.len();
        for i in 0..len {
            let e = self.elements[i].clone();
            pre_increment = match e {
                Elem::Word(_, _) => {
                    if pre_increment != 0 {
                        ans.push(Elem::Increment(pre_increment));
                    }
                    ans.push(e);
                    0
                },
                Elem::Increment(n) => self.preinc_to_unarys(&mut ans, i, n),
                _ => {
                    ans.push(self.elements[i].clone());
                    0
                },
            };
        }

        match pre_increment {
            1  => Err(syntax_error_msg("++")),
            -1 => Err(syntax_error_msg("--")),
            _  => Ok(ans),
        }
    }

    pub fn new() -> ArithmeticExpr {
        ArithmeticExpr {
            text: String::new(),
            elements: vec![],
            paren_stack: vec![],
        }
    }
}
