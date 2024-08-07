//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod calculator;
mod word_manip;

use crate::{ShellCore, Feeder};
use self::calculator::calculate;
use super::word::Word;

#[derive(Debug, Clone)]
enum Elem {
    UnaryOp(String),
    BinaryOp(String),
    Operand(i64),
    ConditionalOp(Box<Option<ArithmeticExpr>>, Box<Option<ArithmeticExpr>>),
    Word(Word, i64), // Word[++, --]
    LeftParen,
    RightParen,
    PlusPlus,
    MinusMinus,
    Increment(i64),
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
            Ok(ans)  => Some(ans.to_string()),
            Err(msg) => {
                eprintln!("sush: {}: {}", &self.text, msg);
                None
            },
        }
    }

    pub fn eval_in_cond(&mut self, core: &mut ShellCore) -> Result<i64, String> {
        let es = match self.decompose_increments() {
            Ok(data)     => data, 
            Err(err_msg) => return Err(err_msg),
        };

        match calculate(&es, core) {
            Ok(ans)      => Ok(ans),
            Err(err_msg) => return Err(err_msg),
        }
    }

    fn inc_dec_to_unarys(&mut self, ans: &mut Vec<Elem>, pos: usize, inc: i64) -> i64 {
        let pm = match inc {
            1  => "+",
            -1 => "-",
            _ => return 0,
        }.to_string();
    
        match (&ans.last(), &self.elements.iter().nth(pos+1)) {
            (_, None)                           => return inc,
            (_, Some(&Elem::Word(_, _))) => return inc,
            (Some(&Elem::Operand(_)), _) => ans.push(Elem::BinaryOp(pm.clone())),
            _                                   => ans.push(Elem::UnaryOp(pm.clone())),
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
                    match pre_increment {
                        1  => ans.push(Elem::PlusPlus),
                        -1 => ans.push(Elem::MinusMinus),
                        _  => {},
                    }

                    ans.push(e);
                    0
                },
                Elem::Increment(n) => self.inc_dec_to_unarys(&mut ans, i, n),
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

    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        let len = feeder.scanner_multiline_blank(core);
        ans.text += &feeder.consume(len);
    }

    fn eat_suffix(feeder: &mut Feeder, ans: &mut Self) -> i64 {
        if feeder.starts_with("++") {
            ans.text += &feeder.consume(2);
            1
        } else if feeder.starts_with("--") {
            ans.text += &feeder.consume(2);
            -1
        } else{
            0
        }
    }

    fn eat_incdec(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("++") {
            ans.text += &feeder.consume(2);
            ans.elements.push( Elem::Increment(1) );
        }else if feeder.starts_with("--") {
            ans.text += &feeder.consume(2);
            ans.elements.push( Elem::Increment(-1) );
        }else {
            return false;
        };
        true
    }

    fn eat_conditional_op(feeder: &mut Feeder,
        ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! feeder.starts_with("?") {
            return false;
        }

        ans.text += &feeder.consume(1);
        let left = Self::parse(feeder, core);
        if left.is_some() {
            ans.text += &left.as_ref().unwrap().text;
        }

        if ! feeder.starts_with(":") {
            ans.elements.push(Elem::ConditionalOp(Box::new(left), Box::new(None)));
            return true;
        }

        ans.text += &feeder.consume(1);
        let right = Self::parse(feeder, core);
        if right.is_some() {
            ans.text += &right.as_ref().unwrap().text;
        }

        ans.elements.push(Elem::ConditionalOp(Box::new(left), Box::new(right)));
        true
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let mut word = match Word::parse(feeder, core, true) {
            Some(w) => {
                ans.text += &w.text;
                w
            },
            _ => return false,
        };

        if let Some(w) = word.make_unquoted_word() {
            if word.text.find('\'').is_none() {
                if let Some(n) = word_manip::parse_as_i64(&w) {
                    ans.elements.push( Elem::Operand(n) );
                    return true;
                }
            }
        }

        Self::eat_blank(feeder, ans, core);

        let suffix = Self::eat_suffix(feeder, ans);
        ans.elements.push( Elem::Word(word, suffix) );
        true
    }

    fn eat_unary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match &ans.elements.last() {
            Some(Elem::Operand(_)) 
            | Some(Elem::Word(_, _)) 
            | Some(Elem::RightParen) => return false,
            _ => {},
        }

        let s = match feeder.scanner_unary_operator(core) {
            0   => return false,
            len => feeder.consume(len),
        };

        ans.text += &s.clone();
        ans.elements.push( Elem::UnaryOp(s) );
        true
    }

    fn eat_paren(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("(") {
            ans.paren_stack.push( '(' );
            ans.elements.push( Elem::LeftParen );
            ans.text += &feeder.consume(1);
            return true;
        }

        if feeder.starts_with(")") {
            if let Some('(') = ans.paren_stack.last() {
                ans.paren_stack.pop();
                ans.elements.push( Elem::RightParen );
                ans.text += &feeder.consume(1);
                return true;
            }
        }

        false
    }

    fn eat_binary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_binary_operator(core);
        if len == 0 {
            return false;
        }

        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( Elem::BinaryOp(s) );
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<ArithmeticExpr> {
        let mut ans = ArithmeticExpr::new();

        loop {
            Self::eat_blank(feeder, &mut ans, core);

            if feeder.starts_with(":") {
                break;
            }

            if Self::eat_conditional_op(feeder, &mut ans, core) 
            || Self::eat_incdec(feeder, &mut ans) 
            || Self::eat_unary_operator(feeder, &mut ans, core)
            || Self::eat_paren(feeder, &mut ans)
            || Self::eat_binary_operator(feeder, &mut ans, core)
            || Self::eat_word(feeder, &mut ans, core) { 
                continue;
            }

            if feeder.len() != 0 || ! feeder.feed_additional_line(core) {
                break;
            }
        }

        return Some(ans);
    }
}
