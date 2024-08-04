//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod calculator;

use crate::{ShellCore, Feeder};
use self::calculator::calculate;
use super::word::Word;

#[derive(Debug, Clone)]
enum CalcElement {
    UnaryOp(String),
    BinaryOp(String),
    Operand(i64),
    Word(Word, i64), //i64: ++:1 --:-1
    LeftParen,
    RightParen,
    Increment(i64),
}

#[derive(Debug, Clone)]
pub struct Calc {
    pub text: String,
    elements: Vec<CalcElement>,
    paren_stack: Vec<char>,
}

fn syntax_error_msg(token: &str) -> String {
    format!("{0}: syntax error: operand expected (error token is \"{0}\")", token)
}

fn recursion_error(token: &str) -> String {
    format!("{0}: expression recursion level exceeded (error token is \"{0}\")", token)
}

fn is_name(s: &str, core: &mut ShellCore) -> bool {
    let mut f = Feeder::new(s);
    s.len() > 0 && f.scanner_name(core) == s.len()
}

impl Calc {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<String> {
        let es = match self.words_to_operands(core) {
            Ok(data)     => data, 
            Err(err_msg) => {
                eprintln!("sush: {}", err_msg);
                return None;
            },
        };

        match calculate(&es) {
            Ok(ans)  => Some(ans),
            Err(msg) => {
                eprintln!("sush: {}: {}", &self.text, msg);
                None
            },
        }
    }

    fn increment_variable(name: &str, core: &mut ShellCore, inc: i64, pre: bool) -> Result<i64, String> {
        if name.len() == 0 {
            return Ok(0);
        }
        if ! is_name(name, core) {
            if inc != 0 && ! pre {
                return Err(syntax_error_msg(name));
            }
            return Self::value_to_num(&name, core);
        }

        let num_i64 = match Self::value_to_num(&name, core) {
            Ok(n)        => n,
            Err(err_msg) => return Err(err_msg), 
        };
        
        core.data.set_param(name, &(num_i64 + inc).to_string());

        match pre {
            true  => Ok(num_i64+inc),
            false => Ok(num_i64),
        }
    }

    fn word_to_operand(w: &Word, pre_increment: i64, post_increment: i64,
                       core: &mut ShellCore) -> Result<CalcElement, String> {
        if w.text.find('\'').is_some() {
            return Err(syntax_error_msg(&w.text));
        }

        let name = match w.eval_as_value(core) {
            Some(v) => v, 
            None => return Err(format!("{}: wrong substitution", &w.text)),
        };

        let res = match pre_increment {
            0 => Self::increment_variable(&name, core, post_increment, false),
            _ => Self::increment_variable(&name, core, pre_increment, true),
        };

        match res {
            Ok(n)  => return Ok(CalcElement::Operand(n)),
            Err(e) => return Err(e),
        }
    }

    fn inc_dec_to_unarys(&mut self, ans: &mut Vec<CalcElement>, pos: usize, inc: i64) -> i64 {
        let pm = match inc {
            1  => "+",
            -1 => "-",
            _ => return 0,
        }.to_string();
    
        match (&ans.last(), &self.elements.iter().nth(pos+1)) {
            (_, None)                           => return inc,
            (_, Some(&CalcElement::Word(_, _))) => return inc,
            (Some(&CalcElement::Operand(_)), _) => ans.push(CalcElement::BinaryOp(pm.clone())),
            _                                   => ans.push(CalcElement::UnaryOp(pm.clone())),
        }
        ans.push(CalcElement::UnaryOp(pm));
        0
    }

    fn words_to_operands(&mut self, core: &mut ShellCore) -> Result<Vec<CalcElement>, String> {
        let mut ans = vec![];
        let mut pre_increment = 0;

        let len = self.elements.len();
        for i in 0..len {
            let e = self.elements[i].clone();
            pre_increment = match e {
                CalcElement::Word(w, post_increment) => {
                    if let Some(CalcElement::Operand(_)) = ans.last() {
                        return Err(syntax_error_msg(&w.text));
                    }

                    if pre_increment != 0 && post_increment != 0 {
                        return Err(syntax_error_msg(&w.text));
                    }

                    match Self::word_to_operand(&w, pre_increment, post_increment, core) {
                        Ok(n)    => ans.push(n),
                        Err(msg) => return Err(msg),
                    }
                    0
                },
                CalcElement::Increment(n) => self.inc_dec_to_unarys(&mut ans, i, n),
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

    fn value_to_num(name: &str, core: &mut ShellCore) -> Result<i64, String> {
        let mut converted_name = name.to_string();

        const RESOLVE_LIMIT: i32 = 10000;

        for i in 0..RESOLVE_LIMIT {
            let mut f = Feeder::new(&converted_name);
            if converted_name.len() > 0 && f.scanner_name(core) == converted_name.len() {
                converted_name = core.data.get_param(&converted_name);
            }else{
                break;
            }

            if i == RESOLVE_LIMIT - 1 {
                return Err(recursion_error(name));
            }
        }

        if let Ok(n) = converted_name.parse::<i64>() {
            Ok( n )
        }else if converted_name == "" {
            Ok( 0 )
        }else if is_name(&converted_name, core) {
            Ok( 0 )
        }else{
            Err(syntax_error_msg(name))
        }
    }

    pub fn new() -> Calc {
        Calc {
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
            ans.elements.push( CalcElement::Increment(1) );
        }else if feeder.starts_with("--") {
            ans.text += &feeder.consume(2);
            ans.elements.push( CalcElement::Increment(-1) );
        }else {
            return false;
        };
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

        if let Some(n) = word.eval_as_operand_literal() {
            ans.elements.push( CalcElement::Operand(n) );
            return true;
        }

        Self::eat_blank(feeder, ans, core);

        let suffix = Self::eat_suffix(feeder, ans);
        ans.elements.push( CalcElement::Word(word, suffix) );
        true
    }

    fn eat_unary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match &ans.elements.last() {
            Some(CalcElement::Operand(_)) => return false,
            Some(CalcElement::Word(_, _)) => return false,
            _ => {},
        }

        let len = feeder.scanner_calc_operator(core);
        if len == 0 {
            return false;
        }

        let s = match feeder.starts_with("+") || feeder.starts_with("-") {
            true  => feeder.consume(1),
            false => return false,
        };

        ans.text += &s.clone();
        ans.elements.push( CalcElement::UnaryOp(s) );
        true
    }

    fn eat_paren(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("(") {
            ans.paren_stack.push( '(' );
            ans.elements.push( CalcElement::LeftParen );
            ans.text += &feeder.consume(1);
            return true;
        }

        if feeder.starts_with(")") {
            if let Some('(') = ans.paren_stack.last() {
                ans.paren_stack.pop();
                ans.elements.push( CalcElement::RightParen );
                ans.text += &feeder.consume(1);
                return true;
            }
        }

        false
    }

    fn eat_binary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_calc_operator(core);
        if len == 0 {
            return false;
        }

        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( CalcElement::BinaryOp(s) );
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Calc> {
        let mut ans = Calc::new();

        loop {
            Self::eat_blank(feeder, &mut ans, core);
            if Self::eat_incdec(feeder, &mut ans) 
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

        match feeder.starts_with("))") {
            true  => Some(ans),
            false => None,
        }
    }
}
