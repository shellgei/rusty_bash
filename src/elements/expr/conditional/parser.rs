//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::simple::SimpleSubword;
use crate::elements::word::Word;
use super::{CondElem, ConditionalExpr};

impl ConditionalExpr {
    fn eat_word(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if feeder.starts_with("]]")
        || feeder.starts_with(")")
        || feeder.starts_with("(") {
            return false;
        }

        match Word::parse(feeder, core, false) {
            Ok(Some(w)) => {
                ans.text += &w.text.clone();
                ans.elements.push(CondElem::Word(w));

                true
            },
            _ => false
        }
    }

    fn eat_compare_op(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> String {
        let len = feeder.scanner_test_compare_op(core);
        if len == 0 {
            return "".to_string();
        }

        let opt = feeder.consume(len);
        ans.text += &opt.clone();
        ans.elements.push(CondElem::BinaryOp(opt.clone()));

        opt
    }

    fn eat_subwords(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Word {
        let mut word = Word::default();
        while ! feeder.starts_with(" ") {
            if let Ok(Some(sw)) = subword::parse(feeder, core) {
                ans.text += sw.get_text();
                word.text += sw.get_text();
                word.subwords.push(sw);
                continue;
            }

            let len = feeder.scanner_regex_symbol();
            if len == 0 {
                break;
            }

            let symbol = feeder.consume(len);
            ans.text += &symbol.clone();
            word.subwords.push( Box::new( SimpleSubword { text: symbol } ) );
        }

        word
    }

    fn eat_regex(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! Self::eat_blank(feeder, ans, core) {
            return false;
        }

        let w = Self::eat_subwords(feeder, ans, core);
        ans.elements.push( CondElem::Regex(w) );
        true
    }

    fn eat_file_check_option(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_test_check_option(core);
        if len == 0 {
            return false;
        }

        let opt = feeder.consume(len);
        ans.text += &opt.clone();
        ans.elements.push(CondElem::UnaryOp(opt));

        true
    }

    fn eat_not_and_or(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("!") {
            ans.text += &feeder.consume(1);
            ans.elements.push( CondElem::Not );
            return true;
        }
        if feeder.starts_with("&&") {
            ans.text += &feeder.consume(2);
            ans.elements.push( CondElem::And );
            return true;
        }
        if feeder.starts_with("||") {
            ans.text += &feeder.consume(2);
            ans.elements.push( CondElem::Or );
            return true;
        }

        false
    }

    fn eat_paren(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(e) = ans.elements.last() {
            match e {
                CondElem::UnaryOp(_) => {
                    return false
                },
                _ => {},
            }
        }

        if ! feeder.starts_with("(") {
            return false;
        }

        ans.text += &feeder.consume(1);

        let expr = match Self::parse(feeder, core) {
            Some(e) => e,
            None    => return false,
        };

        if ! feeder.starts_with(")") {
            return false;
        }

        ans.text += &expr.text.clone();
        ans.elements.push( CondElem::InParen(expr) );
        ans.text += &feeder.consume(1);
        true
    }

    fn eat_blank(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        match feeder.scanner_blank(core) {
            0 => false,
            n => {
                ans.text += &feeder.consume(n);
                true
            },
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        let mut ans = Self::default();

        loop {
            Self::eat_blank(feeder, &mut ans, core);
            if feeder.starts_with("\n"){
                ans.text += &feeder.consume(1);
                continue;
            }
            if feeder.len() == 0 {
                if ! feeder.feed_additional_line(core).is_ok() {
                    return None;
                }
                continue;
            }

            if feeder.starts_with("]]")
            || feeder.starts_with(")") {
                if ans.elements.is_empty() {
                    return None;
                }

                ans.elements.push(CondElem::And);
                return Some(ans);
            }

            if Self::eat_paren(feeder, &mut ans, core) {
                continue;
            }

            match Self::eat_compare_op(feeder, &mut ans, core).as_ref() {
                "" => {},
                "=~" => {
                    match Self::eat_regex(feeder, &mut ans, core) {
                        false => return None,
                        true  => continue,
                    }
                },
                _ => continue,
            }
 
            if Self::eat_file_check_option(feeder, &mut ans, core)
            || Self::eat_not_and_or(feeder, &mut ans) 
            || Self::eat_word(feeder, &mut ans, core) {
                continue;
            }

            break;
        }
        None
    }
}
