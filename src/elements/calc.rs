//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod calculator;

use crate::{ShellCore, Feeder};
use self::calculator::calculate;

#[derive(Debug, Clone)]
enum CalcElement {
    Op(String),
    //Var(Box<dyn Subword>),
    Num(i64),
}

#[derive(Debug, Clone)]
pub struct Calc {
    pub text: String,
    elements: Vec<CalcElement>,
    rev_polish: Vec<CalcElement>,
    paren_stack: Vec<char>,
}

impl Calc {
    pub fn eval(&mut self, _: &mut ShellCore) -> Option<String> {
        match calculate(&self.elements) {
            Some(CalcElement::Num(n)) => return Some(n.to_string()),
            _ => return None,
        }

        None
    }

    fn to_rev_polish(&mut self) {
        self.rev_polish = self.elements.to_vec();
    }

    pub fn new() -> Calc {
        Calc {
            text: String::new(),
            elements: vec![],
            rev_polish: vec![],
            paren_stack: vec![],
        }
    }

    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        let len = feeder.scanner_multiline_blank(core);
        ans.text += &feeder.consume(len);
    }

    fn eat_interger(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_integer(core);
        if len == 0 {
            return false;
        }

        let s = feeder.consume(len);
        ans.text += &s.clone();
        let n = s.parse::<i64>().expect("SUSH INTERNAL ERROR: scanner_integer is wrong");
        ans.elements.push( CalcElement::Num(n) );

        true
    }

    fn eat_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_calc_operator(core);
        if len == 0 {
            return false;
        }

        if feeder.starts_with("(") {
            ans.paren_stack.push( '(' );
        }else if feeder.starts_with(")") {
            match ans.paren_stack.last() {
                Some('(') => {ans.paren_stack.pop();},
                _         => return false,
            }
        }

        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( CalcElement::Op(s) );
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Calc> {
        let mut ans = Calc::new();

        loop {
            Self::eat_blank(feeder, &mut ans, core);
            if Self::eat_interger(feeder, &mut ans, core) 
            || Self::eat_operator(feeder, &mut ans, core) {
                continue;
            }

            if feeder.len() != 0 || ! feeder.feed_additional_line(core) {
                break;
            }
        }

        match feeder.starts_with("))") {
            true  => Some(ans),
            false => None,
        }
    }
}
