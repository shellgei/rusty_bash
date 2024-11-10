//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod offset;
mod alternative;

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;
use crate::elements::subscript::Subscript;
use crate::elements::word::Word;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::utils::glob;
use super::simple::SimpleSubword;

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    pub text: String,
    pub name: String,
    unknown: String,
    subscript: Option<Subscript>,
    has_alternative: bool,
    alternative_symbol: Option<String>,
    alternative_value: Option<Word>,
    num: bool,
    has_offset: bool,
    offset: Option<ArithmeticExpr>,
    has_length: bool,
    length: Option<ArithmeticExpr>,
    has_remove_pattern: bool,
    remove_symbol: String,
    remove_pattern: Option<Word>,
}

fn is_param(s :&String) -> bool {
    if s.len() == 0 {
        return false;
    }

    let first_ch = s.chars().nth(0).unwrap();
    if s.len() == 1 { //special or position param
        if "$?*@#-!_0123456789".find(first_ch) != None {
            return true;
        }
    }
    /* variable */
    if '0' <= first_ch && first_ch <= '9' {
        return s.chars().position(|c| c < '0' || '9' < c) == None;
    }

    let name_c = |c| ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z')
                     || ('0' <= c && c <= '9') || '_' == c;
    s.chars().position(|c| !name_c(c)) == None
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str { &self.text.as_ref() }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        if ! self.check() {
            return false;
        }

        if let Some(sub) = self.subscript.as_mut() {
            if let Some(s) = sub.eval() {
                self.text = match (self.num, s.as_str()) {
                    (true, "@") => core.data.get_array_len(&self.name).to_string(),
                    (true, _)   => core.data.get_array(&self.name, &s).chars().count().to_string(),
                    (false, _)  => core.data.get_array(&self.name, &s),
                };
            }
        }else{
            let value = core.data.get_param(&self.name);
            self.text = match self.num {
                true  => value.chars().count().to_string(),
                false => value.to_string(),
            };
        }

        if self.has_offset {
            match offset::get(self, core) {
                Some(text) => self.text = text,
                None => return false,
            }
        }else if self.has_alternative {
            return self.replace_to_alternative(core);
        }else if self.has_remove_pattern {
            return self.remove(core);
        }

        true
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }

    fn get_alternative_subwords(&self) -> Vec<Box<dyn Subword>> {
        match self.alternative_value.as_ref() {
            Some(w) => w.subwords.to_vec(),
            None    => vec![],
        }
    }
}

impl BracedParam {
    fn check(&mut self) -> bool {
        if self.name.len() == 0 || ! is_param(&self.name) {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }
        if self.unknown.len() > 0 
        && ! self.unknown.starts_with(",") {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }
        true
    }

    fn remove(&mut self, core: &mut ShellCore) -> bool {
       let pattern = match &self.remove_pattern {
           Some(w) => {
               match w.eval_for_case_word(core) {
                   Some(s) => s,
                   None    => return false,
               }
           },
           None => return true,
       };

       let extglob = core.shopts.query("extglob");

       if self.remove_symbol.starts_with("#") {
           let mut length = 0;
           let mut ans_length = 0;

           for ch in self.text.chars() {
               length += ch.len_utf8();
               let s = self.text[0..length].to_string();

               if glob::compare(&s, &pattern, extglob) {
                   ans_length = length;
                   if self.remove_symbol == "#" {
                       break;
                   }
               }
           }

           self.text = self.text[ans_length..].to_string();
           return true;
       }

       if self.remove_symbol.starts_with("%") {
           let mut length = self.text.len();
           let mut ans_length = length;

           for ch in self.text.chars().rev() {
               length -= ch.len_utf8();
               let s = self.text[length..].to_string();

               if glob::compare(&s, &pattern, extglob) {
                   ans_length = length;
                   if self.remove_symbol == "%" {
                       break;
                   }
               }
           }

           self.text = self.text[0..ans_length].to_string();
           return true;
       }

       true
    }

    /*
    fn replace_to_alternative(&mut self, core: &mut ShellCore) -> bool {
        let symbol = match (self.alternative_symbol.as_deref(), self.text.as_ref()) {
            (Some(s), "")   => s,
            (Some("-"), _)  => "-",
            (Some(":+"), _) => ":+",
            (Some("+"), _)  => "+",
            _               => return true,
        };

        let word = match self.alternative_value.as_ref() {
            Some(w) => match w.tilde_and_dollar_expansion(core) {
                Some(w2) => w2,
                None     => return false,
            },
            None => return false,
        };

        let value: String = word.subwords.iter().map(|s| s.get_text()).collect();

        if symbol == "-" {
            self.alternative_value = None;
            self.alternative_symbol = None;
            return true;
        }
        if symbol == "+" {
            if ! core.data.has_value(&self.name) {
                self.alternative_value = None;
                return true;
            }
            self.alternative_value = Some(word);
            return true;
        }
        if symbol == ":-" {
            self.alternative_value = Some(word);
            return true;
        }
        if symbol == ":=" {
            if ! core.data.set_param(&self.name, &value) {
                return false;
            }
            self.alternative_value = None;
            self.text = value;
            return true
        }
        if symbol == ":?" {
            eprintln!("sush: {}: {}", &self.name, &value);
            return false;
        }
        if symbol == ":+" {
            self.alternative_value = match self.text.as_str() {
                "" => None,
                _  => Some(word),
            };
            return true;
        }

        return false;
    }
    */

    fn eat_subscript(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(s) = Subscript::parse(feeder, core) {
            ans.text += &s.text;
            ans.subscript = Some(s);
            return true;
        }

        false
    }

    fn eat_offset(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! feeder.starts_with(":") {
            return false;
        }
        ans.text += &feeder.consume(1);
        ans.has_offset = true;
        ans.offset = match ArithmeticExpr::parse(feeder, core, true) {
            Some(a) => {
                ans.text += &a.text.clone();
                Self::eat_length(feeder, ans, core);
                Some(a)
            },
            None => None,
        };
        true
    }

    fn eat_remove_pattern(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_parameter_remove_symbol();
        if len == 0 {
            return false;
        }

        ans.remove_symbol = feeder.consume(len);
        ans.text += &ans.remove_symbol.clone();
        ans.has_remove_pattern = true;

        ans.remove_pattern = Some(Self::eat_subwords(feeder, ans, core));
        true
    }

    fn eat_length(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        if ! feeder.starts_with(":") {
            return;
        }
        ans.text += &feeder.consume(1);
        ans.has_length = true;
        ans.length = match ArithmeticExpr::parse(feeder, core, true) {
            Some(a) => {
                ans.text += &a.text.clone();
                Some(a)
            },
            None => None,
        };
    }

    fn eat_alternative_value(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_parameter_alternative_symbol();
        if num == 0 {
            return false;
        }

        ans.has_alternative = true;
        let symbol = feeder.consume(num);
        ans.alternative_symbol = Some(symbol.clone());
        ans.text += &symbol;

        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        ans.alternative_value = Some(Self::eat_subwords(feeder, ans, core));
        true
    }

    fn eat_blank(len: usize, feeder: &mut Feeder, ans: &mut Self, word: &mut Word) {
        let blank = feeder.consume(len);
        let sw = Box::new(SimpleSubword{ text: blank.clone() });
        word.subwords.push(sw);
        ans.text += &blank.clone();
    }

    fn eat_subwords(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> Word {
        let mut word = Word::default();
        while ! feeder.starts_with("}") {
            if let Some(sw) = subword::parse(feeder, core) {
                ans.text += sw.get_text();
                word.text += sw.get_text();
                word.subwords.push(sw);
            }

            if feeder.starts_with("\n") {
                Self::eat_blank(1, feeder, ans, &mut word);
                feeder.feed_additional_line(core);
            }

            let num = feeder.scanner_blank(core);
            if num != 0 {
                Self::eat_blank(num, feeder, ans, &mut word);
            }
        }

        word
    }

    fn eat_param(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if len != 0 {
            ans.name = feeder.consume(len);
            ans.text += &ans.name;
            return true;
        }

        let len = feeder.scanner_special_and_positional_param();
        if len != 0 {
            ans.name = feeder.consume(len);
            ans.text += &ans.name;
            return true;
        }

        feeder.starts_with("}")
    }

    fn eat_unknown(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) {
        if feeder.len() == 0 {
            feeder.feed_additional_line(core);
        }

        let unknown = match feeder.starts_with("\\}") {
            true  => feeder.consume(2),
            false => feeder.consume(1),
        };

        ans.unknown += &unknown.clone();
        ans.text += &unknown;
        return;
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BracedParam> {
        if ! feeder.starts_with("${") {
            return None;
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(2);

        if feeder.starts_with("#") && ! feeder.starts_with("#}") {
            ans.num = true;
            ans.text += &feeder.consume(1);
        }

        if Self::eat_param(feeder, &mut ans, core) {
            Self::eat_subscript(feeder, &mut ans, core);
            let _ = Self::eat_alternative_value(feeder, &mut ans, core) 
                 || Self::eat_offset(feeder, &mut ans, core)
                 || Self::eat_remove_pattern(feeder, &mut ans, core);
        }

        while ! feeder.starts_with("}") {
            Self::eat_unknown(feeder, &mut ans, core);
        }

        ans.text += &feeder.consume(1);
        Some(ans)
    }
}
