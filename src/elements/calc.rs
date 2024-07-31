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
    Num(i64),
    Name(String, Box<CalcElement>), //CalcElement: PlusPlus or MinusMinus
    Word(Word, Box<CalcElement>),
    LeftParen,
    RightParen,
    PlusPlus,
    MinusMinus,
    Noop,
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

impl Calc {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<String> {
        let es = match self.evaluate_elems(core) {
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

    fn evaluate_name(name: &str, prev_inc: i64, after_inc: CalcElement, core: &mut ShellCore)
                                                      -> Result<CalcElement, String> {
        let mut num;
        let ans = match Self::value_to_num(name, core) {
            Ok(n) => {
                num = n;
                CalcElement::Num(n+prev_inc)
            },
            Err(err_msg) => return Err(err_msg), 
        };

        num += prev_inc;
        match after_inc {
            CalcElement::PlusPlus   => num += 1,
            CalcElement::MinusMinus => num -= 1,
            _ => {},
        }

        core.data.set_param(&name, &num.to_string());
        Ok(ans)
    }

    fn evaluate_elems(&mut self, core: &mut ShellCore) -> Result<Vec<CalcElement>, String> {
        let mut ans = vec![];
        let mut next_inc: i64 = 0;

        for e in &self.elements {
            match e {
                CalcElement::Name(s, inc) => {
                    match Self::evaluate_name(s, next_inc, *inc.clone(), core) {
                        Ok(e)    => ans.push(e),
                        Err(msg) => return Err(msg),
                    }
                },
                CalcElement::Word(w, inc) => {
                    if w.text.find('\'').is_some() {
                        return Err(syntax_error_msg(&w.text));
                    }

                    let val = match w.eval_as_value(core) {
                        Some(v) => v, 
                        None => return Err(format!("{}: wrong substitution", &self.text)),
                    };

                    match Self::evaluate_name(&val, next_inc, *inc.clone(), core) {
                        Ok(e)    => ans.push(e),
                        Err(msg) => return Err(msg),
                    }
                },
                _ => ans.push(e.clone()),
            }

            next_inc = match e {
                CalcElement::PlusPlus => 1,
                CalcElement::MinusMinus => -1,
                _ => 0, 
            };
        }

        Ok(ans)
    }

    fn value_to_num(name: &str, core: &mut ShellCore) -> Result<i64, String> {
        let mut converted_name = name.to_string();

        loop {
            let mut f = Feeder::new(&converted_name);
            if converted_name.len() > 0 && f.scanner_name(core) == converted_name.len() {
                converted_name = core.data.get_param(&converted_name);
            }else{
                break;
            }
        }

        if let Ok(n) = converted_name.parse::<i64>() {
            Ok( n )
        }else if converted_name == "" {
            Ok( 0 )
        }else if converted_name.find('\'').is_none() {
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

    fn eat_integer(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match &ans.elements.last() {
            Some(CalcElement::Num(_)) => return false,
            _ => {},
        }

        let len = feeder.scanner_nonnegative_integer(core);
        if len == 0 {
            return false;
        }

        let n = match feeder.refer(len).parse::<i64>() {
            Ok(n)  => n, 
            Err(_) => return false,
        };

        ans.inc_dec_to_unarys();
        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( CalcElement::Num(n) );
        true
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return false;
        }

        let s = feeder.consume(len);
        ans.text += &s;
        Self::eat_blank(feeder, ans, core);

        let suffix = if feeder.starts_with("++") {
            ans.text += &feeder.consume(2);
            Box::new(CalcElement::PlusPlus)
        } else if feeder.starts_with("--") {
            ans.text += &feeder.consume(2);
            Box::new(CalcElement::MinusMinus)
        } else{
            Box::new(CalcElement::Noop)
        };
        ans.elements.push( CalcElement::Name(s.clone(), suffix) );

        true
    }

    fn eat_incdec(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("++") {
            ans.text += &feeder.consume(2);
            ans.elements.push( CalcElement::PlusPlus );
        }else if feeder.starts_with("--") {
            ans.text += &feeder.consume(2);
            ans.elements.push( CalcElement::MinusMinus );
        }else {
            return false;
        };
        true
    }

    fn inc_dec_to_unarys(&mut self) {
        let pm = match self.elements.last() {
            Some(CalcElement::PlusPlus) => "+",
            Some(CalcElement::MinusMinus) => "-",
            _ => return,
        }.to_string();

        self.elements.pop();

        match self.elements.last() {
            None |
            Some(CalcElement::UnaryOp(_)) |
            Some(CalcElement::BinaryOp(_)) |
            Some(CalcElement::LeftParen) 
               => self.elements.push(CalcElement::UnaryOp(pm.clone())),
            _  => self.elements.push(CalcElement::BinaryOp(pm.clone())),
        }
        self.elements.push(CalcElement::UnaryOp(pm));
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let mut word = match Word::parse(feeder, core) {
            Some(w) => {
                ans.text += &w.text;
                w
            },
            _ => return false,
        };

        let size = word.subwords.len();
        if size > 2 {
            if (word.subwords[size-1].get_text() == "+" && word.subwords[size-2].get_text() == "+" )
            || (word.subwords[size-1].get_text() == "-" && word.subwords[size-2].get_text() == "-" ) {
                word.subwords.pop();
                word.subwords.pop();
                word.text.pop();
                match word.text.pop() {
                    Some('+') => ans.elements.push( CalcElement::Word(word, Box::new(CalcElement::PlusPlus)) ),
                    Some('-') => ans.elements.push( CalcElement::Word(word, Box::new(CalcElement::MinusMinus)) ),
                    _ => {},
                }
                return true;
            }
        }

        Self::eat_blank(feeder, ans, core);

        let suffix = if feeder.starts_with("++") {
            ans.text += &feeder.consume(2);
            Box::new(CalcElement::PlusPlus)
        } else if feeder.starts_with("--") {
            ans.text += &feeder.consume(2);
            Box::new(CalcElement::MinusMinus)
        } else{
            Box::new(CalcElement::Noop)
        };
        ans.elements.push( CalcElement::Word(word, suffix) );
        true
    }

    fn eat_unary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match &ans.elements.last() {
            Some(CalcElement::Num(_)) => return false,
            Some(CalcElement::Name(_, _)) => return false,
            Some(CalcElement::Word(_, _)) => return false,
            _ => {},
        }

        let len = feeder.scanner_calc_operator(core);
        if len == 0 {
            return false;
        }

        let s = if feeder.starts_with("+") || feeder.starts_with("-") {
            feeder.consume(1)
        }else{
            return false
        };

        ans.inc_dec_to_unarys();
        ans.text += &s.clone();
        ans.elements.push( CalcElement::UnaryOp(s) );
        true
    }

    fn eat_paren(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("(") {
            ans.inc_dec_to_unarys();
            ans.paren_stack.push( '(' );
            ans.elements.push( CalcElement::LeftParen );
            ans.text += &feeder.consume(1);
            return true;
        }

        if feeder.starts_with(")") {
            if let Some('(') = ans.paren_stack.last() {
                ans.inc_dec_to_unarys();
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

        ans.inc_dec_to_unarys();
        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( CalcElement::BinaryOp(s) );
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Calc> {
        let mut ans = Calc::new();

        loop {
            Self::eat_blank(feeder, &mut ans, core);
            if Self::eat_name(feeder, &mut ans, core) 
            || Self::eat_incdec(feeder, &mut ans) 
            || Self::eat_unary_operator(feeder, &mut ans, core)
            || Self::eat_paren(feeder, &mut ans)
            || Self::eat_binary_operator(feeder, &mut ans, core)
            || Self::eat_integer(feeder, &mut ans, core) 
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
