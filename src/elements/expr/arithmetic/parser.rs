//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::parse::ParseError;
use crate::elements::subscript::Subscript;
use crate::elements::word::{Word, WordMode};
use super::{ArithmeticExpr, ArithElem};
use super::elem::{int, float};

impl ArithmeticExpr {
    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        let len = feeder.scanner_multiline_blank(core);
        ans.text += &feeder.consume(len);
    }

    fn eat_space(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        let len = feeder.scanner_multiline_blank(core);
        if len == 0 {
            return;
        }
        let sp = feeder.consume(len);
        ans.elements.push(ArithElem::Space(sp.clone()));
        ans.text += &sp;
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
        if feeder.starts_with("++") && ! feeder.starts_with("+++") {
            ans.text += &feeder.consume(2);
            ans.elements.push( ArithElem::Increment(1) );
        }else if feeder.starts_with("--") && ! feeder.starts_with("---") {
            ans.text += &feeder.consume(2);
            ans.elements.push( ArithElem::Increment(-1) );
        }else {
            return false;
        };
        true
    }

    fn eat_symbol(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_withs(&["?", ":"]) {
            let symbol = feeder.consume(1);
            ans.text += &symbol.clone();
            ans.elements.push( ArithElem::Symbol(symbol));
            return true;
        }

        false
    }

    fn eat_num(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_arith_number(core);
        if len == 0 {
            return false;
        }

        let w = feeder.consume(len);
        ans.text += &w.clone();
        if let Ok(n) = int::parse(&w) {
            ans.elements.push( ArithElem::Integer(n) );
            return true;
        }else if let Ok(f) = float::parse(&w) {
            ans.elements.push( ArithElem::Float(f) );
            return true;
        }

        Self::eat_blank(feeder, ans, core);
        let suffix = Self::eat_suffix(feeder, ans);
        ans.elements.push( ArithElem::Word(Word::from(&w), suffix) );
        true
    }

    fn eat_conditional_op(feeder: &mut Feeder,
        ans: &mut Self, core: &mut ShellCore) -> Result<bool, ParseError> {
        if ! feeder.starts_with("?") {
            return Ok(false);
        }
 
        ans.text += &feeder.consume(1);
        let left = Self::parse_after_eval(feeder, core, "?")?;
        if left.is_some() {
            ans.text += &left.as_ref().unwrap().text;
        }

        if ! feeder.starts_with(":") {
            ans.elements.push(ArithElem::Ternary(Box::new(left), Box::new(None)));
            return Ok(true);
        }

        ans.text += &feeder.consume(1);
        let right = Self::parse_after_eval(feeder, core, ":")?;
        if right.is_some() {
            ans.text += &right.as_ref().unwrap().text;
        }

        ans.elements.push(ArithElem::Ternary(Box::new(left), Box::new(right)));
        Ok(true)
    }

    fn eat_array_elem(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Result<bool, ParseError> {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return Ok(false);
        }

        let name = &feeder.consume(len);
        ans.text += &name.clone();

        if let Some(s) = Subscript::parse(feeder, core)? {
            ans.text += &s.text.clone();
            Self::eat_blank(feeder, ans, core);
            let suffix = Self::eat_suffix(feeder, ans);
            ans.elements.push( ArithElem::ArrayElem(name.clone(), s, suffix) );
        }else{
            Self::eat_blank(feeder, ans, core);
            let suffix = Self::eat_suffix(feeder, ans);
            ans.elements.push( ArithElem::Word(Word::from(name), suffix) );
        };

        Ok(true)
    }

    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(c) = feeder.nth(0) {
            if "+=-!~*/%<>&|,?^".contains(c) {
                return false;
            }
        }

        let mut word = match Word::parse(feeder, core, Some(WordMode::Arithmetric)) {
            Ok(Some(w)) => w,
            _       => return false,
        };

        if word.text.starts_with("\"") 
        && word.text.ends_with("\"")
        && word.text.len() >= 2 {
            feeder.replace(0, &word.text[1..word.text.len()-1]);
            return true;
        }

        ans.text += &word.text.clone();

        if let Some(w) = word.make_unquoted_word() {
            if word.text.find('\'').is_none() {
                if let Ok(n) = int::parse(&w) {
                    ans.elements.push( ArithElem::Integer(n) );
                    return true;
                }else if let Ok(f) = float::parse(&w) {
                    ans.elements.push( ArithElem::Float(f) );
                    return true;
                }
            }
        }

        Self::eat_blank(feeder, ans, core);
        let suffix = Self::eat_suffix(feeder, ans);
        ans.elements.push( ArithElem::Word(word, suffix) );
        true
    }

    fn eat_output_format(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_math_output_format(core);
        if len == 0 {
            return false;
        }

        let mut s = feeder.consume(len);
        ans.text += &s.clone();
        ans.hide_base = s.find("##").is_some();
        s.retain(|c| '0' <= c && c <= '9');
        ans.output_base = s;
        true
    }

    fn eat_unary_operator(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match &ans.elements.last() {
            Some(ArithElem::Integer(_)) 
            | Some(ArithElem::Float(_)) 
            | Some(ArithElem::ArrayElem(_, _, _)) 
            | Some(ArithElem::Word(_, _)) 
            | Some(ArithElem::InParen(_)) => return false,
            _ => {},
        }

        let s = match feeder.scanner_unary_operator(core) {
            0   => return false,
            len => feeder.consume(len),
        };

        ans.text += &s.clone();
        ans.elements.push( ArithElem::UnaryOp(s) );
        true
    }

    fn eat_paren_internal(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Self) -> Result<bool, ParseError> {
        if ! feeder.starts_with("(") {
            return Ok(false);
        }

        ans.text += &feeder.consume(1);
        let arith = Self::parse_after_eval(feeder, core, "(")?;
        if arith.is_none() || ! feeder.starts_with(")") {
            return Ok(false);
        }

        ans.text += &arith.as_ref().unwrap().text;
        ans.elements.push( ArithElem::InParen(arith.unwrap()) );

        ans.text += &feeder.consume(1);
        return Ok(true);
    }

    fn eat_paren(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut Self) -> Result<bool, ParseError> {
        if ! feeder.starts_with("(") {
            return Ok(false);
        }

        let paren = feeder.consume(1);
        ans.text += &paren.clone();
        ans.elements.push(ArithElem::Symbol(paren));
        let arith = Self::parse(feeder, core, true, "(")?;
        if arith.is_none() || ! feeder.starts_with(")") {
            return Ok(false);
        }
        ans.text += &arith.as_ref().unwrap().text;
        ans.elements.append(&mut arith.unwrap().elements);

        //ans.elements.push( ArithElem::InParen(arith.unwrap()) );

        let paren = feeder.consume(1);
        ans.text += &paren.clone();
        ans.elements.push(ArithElem::Symbol(paren));
        return Ok(true);
    }

    fn eat_binary_operator(feeder: &mut Feeder, ans: &mut Self,
                           core: &mut ShellCore, left: &str) -> bool {
        let len = feeder.scanner_binary_operator(core);
        if len == 0 {
            return false;
        }

        if left == ":" 
        && ! feeder.starts_with("==")
        && feeder.scanner_substitution(core) > 0 {
                return false;
        }

        let s = feeder.consume(len);
        ans.text += &s.clone();
        ans.elements.push( ArithElem::BinaryOp(s) );
        true
    }

    pub fn parse_after_eval(feeder: &mut Feeder, core: &mut ShellCore, left: &str)
        -> Result<Option<Self>, ParseError> {
        let mut ans = ArithmeticExpr::new();

        loop {
            Self::eat_blank(feeder, &mut ans, core);

            if left == "[" && feeder.starts_with("]") 
            || left == "?" && feeder.starts_with(":")
            || left == ":" && ( feeder.starts_with("]") || feeder.starts_with(":") ) {
                break;
            }

            if Self::eat_output_format(feeder, &mut ans, core) 
            || Self::eat_conditional_op(feeder, &mut ans, core)?
            || Self::eat_incdec(feeder, &mut ans) 
            || Self::eat_unary_operator(feeder, &mut ans, core)
            || Self::eat_paren_internal(feeder, core, &mut ans)?
            || Self::eat_binary_operator(feeder, &mut ans, core, left)
            || Self::eat_array_elem(feeder, &mut ans, core)?
            || Self::eat_num(feeder, &mut ans, core)
            || Self::eat_word(feeder, &mut ans, core) { 
                continue;
            }

            break;
        }
        return Ok(Some(ans));
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore, addline: bool, left: &str)
        -> Result<Option<Self>, ParseError> {
        let mut ans = ArithmeticExpr::new();

        loop {
            Self::eat_space(feeder, &mut ans, core);

            if left == "[" && feeder.starts_with("]") 
            || left == "?" && feeder.starts_with(":")
            || left == ":" && ( feeder.starts_with("]") || feeder.starts_with(":") ) {
                break;
            }

            if Self::eat_output_format(feeder, &mut ans, core) 
            || Self::eat_symbol(feeder, &mut ans)
            || Self::eat_unary_operator(feeder, &mut ans, core)
            || Self::eat_paren(feeder, core, &mut ans)?
            || Self::eat_binary_operator(feeder, &mut ans, core, left)
            || Self::eat_array_elem(feeder, &mut ans, core)?
            || Self::eat_word(feeder, &mut ans, core) { 
                continue;
            }

            if ! addline || feeder.len() != 0 || ! feeder.feed_additional_line(core).is_ok() {
                break;
            }
        }
        return Ok(Some(ans));
    }
}
