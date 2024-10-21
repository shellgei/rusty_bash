//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;
use crate::elements::subscript::Subscript;
use crate::elements::word::Word;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use super::simple::SimpleSubword;

#[derive(Debug, Clone)]
pub struct BracedParam {
    pub text: String,
    pub name: String,
    pub unknown: String,
    pub subscript: Option<Subscript>,
    pub alternative_symbol: Option<String>,
    pub alternative_value: Option<Word>,
    pub num: bool,
    has_offset: bool,
    offset: Option<ArithmeticExpr>,
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
        if self.name.len() == 0 || ! is_param(&self.name) {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }
        if self.unknown.len() > 0 
        && ! self.unknown.starts_with(",") {
            eprintln!("sush: {}: bad substitution", &self.text);
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
            return self.offset(core);
        }

        match self.alternative_symbol.as_deref() {
            Some("-") => {
                self.alternative_value = None;
                self.alternative_symbol = None;
                return true;
            },
            Some(s) => if s == ":+" || s == "+" || self.text == "" {
                return self.replace_to_alternative(core);
            },
            _ => {},
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
    fn new() -> BracedParam {
        BracedParam {
            text: String::new(),
            name: String::new(),
            unknown: String::new(),
            subscript: None,
            alternative_symbol: None,
            alternative_value: None,
            num: false,
            has_offset: false,
            offset: None,
        }
    }

    fn offset(&mut self, core: &mut ShellCore) -> bool {
        let mut offset = match self.offset.clone() {
            None => {
                eprintln!("sush: {}: bad substitution", &self.text);
                return false;
            },
            Some(ofs) => ofs,
        };

        if offset.text == "" {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }

        let num_str = match offset.eval(core) {
            None => return false,
            Some(s) => s,
        };

        match num_str.parse::<usize>() {
            Ok(n) => self.text = self.text.chars().enumerate()
                                     .filter(|(i, _)| i >= &n)
                                     .map(|(_, c)| c).collect(),
            _ => return false,
        }
        true
    }

    fn replace_to_alternative(&mut self, core: &mut ShellCore) -> bool {
        let symbol = match self.alternative_symbol.as_ref() {
            Some(s) => s,
            None    => return true,
        };

        let word = match self.alternative_value.as_ref() {
            Some(w) => match w.tilde_and_dollar_expansion(core) {
                            Some(w2) => w2,
                            None     => return false,
                        },
            None    => return false,
        };

        let value: String = word.subwords.iter().map(|s| s.get_text()).collect();

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
            core.data.set_param(&self.name, &value);
            self.alternative_value = None;
            self.text = value;
            return true;
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

    fn eat_subscript(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(s) = Subscript::parse(feeder, core) {
            ans.text += &s.text;
            ans.subscript = Some(s);
            return true;
        }

        false
    }

    fn push_alternative_subword(len: usize, feeder: &mut Feeder, ans: &mut Self, word: &mut Word) {
        let blank = feeder.consume(len);
        let sw = Box::new(SimpleSubword{ text: blank.clone() });
        word.subwords.push(sw);
        ans.text += &blank.clone();
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
                Some(a)
            },
            None => None,
        };
        true
    }

    fn eat_alternative_value(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_parameter_alternative_symbol();
        if num == 0 {
            return false;
        }
        let symbol = feeder.consume(num);
        ans.alternative_symbol = Some(symbol.clone());
        ans.text += &symbol;

        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        let mut word = Word::new();

        while ! feeder.starts_with("}") {
            if let Some(sw) = subword::parse(feeder, core) {
                ans.text += sw.get_text();
                word.text += sw.get_text();
                word.subwords.push(sw);
            }

            if feeder.starts_with("\n") {
                Self::push_alternative_subword(1, feeder, ans, &mut word);
                feeder.feed_additional_line(core);
            }

            let num = feeder.scanner_blank(core);
            if num != 0 {
                Self::push_alternative_subword(num, feeder, ans, &mut word);
            }
        }

        ans.alternative_value = Some(word);

        true
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
        let mut ans = Self::new();
        ans.text += &feeder.consume(2);

        if feeder.starts_with("#") && ! feeder.starts_with("#}") {
            ans.num = true;
            ans.text += &feeder.consume(1);
        }

        if Self::eat_param(feeder, &mut ans, core) {
            Self::eat_subscript(feeder, &mut ans, core);
            let _ = Self::eat_alternative_value(feeder, &mut ans, core) 
                 || Self::eat_offset(feeder, &mut ans, core);
        }

        while ! feeder.starts_with("}") {
            Self::eat_unknown(feeder, &mut ans, core);
        }

        ans.text += &feeder.consume(1);
        Some(ans)
    }
}
