//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;
use crate::elements::subscript::Subscript;
use crate::elements::word::Word;
use super::simple::SimpleSubword;

#[derive(Debug, Clone)]
pub struct BracedParam {
    pub text: String,
    pub name: String,
    pub unknown: String,
    pub subscript: Option<Subscript>,
    pub default_symbol: String,
    pub default_value: Word,
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
        if self.name.len() == 0 
        || ! is_param(&self.name)
        || ( self.unknown.len() > 0 && ! self.unknown.starts_with("-") ) {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }

        if let Some(sub) = self.subscript.as_mut() {
            if let Some(s) = sub.eval() {
                self.text = core.data.get_array(&self.name, &s);
            }
        }else{
            let value = core.data.get_param(&self.name);
            self.text = value.to_string();
        }

        if self.text == "" || self.default_symbol == ":+" {
            return self.replace_to_default(core);
        }

        true
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }

    fn substitute_replace(&self) -> Vec<Box<dyn Subword>> {
        if self.default_symbol == "" || self.default_value.subwords.len() == 0 {
            return vec![];
        }

        self.default_value.subwords.to_vec()
    }
}

impl BracedParam {
    fn new() -> BracedParam {
        BracedParam {
            text: String::new(),
            name: String::new(),
            unknown: String::new(),
            subscript: None,
            default_value: Word::new(),
            default_symbol: String::new(),
        }
    }

    fn replace_to_default(&mut self, core: &mut ShellCore) -> bool {
        if self.default_symbol == "" {
            return true;
        }

        self.default_value = match self.default_value.tilde_and_dollar_expansion(core) {
                Some(w) => w,
                _       => return false,
        };

        let value: String = self.default_value.subwords.iter().map(|s| s.get_text()).collect();

        if self.default_symbol == ":-" {
            return true;
        }
        if self.default_symbol == ":=" {
            core.data.set_param(&self.name, &value);
            return true;
        }
        if self.default_symbol == ":?" {
            eprintln!("sush: {}: {}", &self.name, &value);
            return false;
        }

        if self.default_symbol == ":+" {
            if self.text == "" {
                self.default_value.subwords.clear();
            }
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

    fn push_default_subword(len: usize, feeder: &mut Feeder, ans: &mut Self) {
        let blank = feeder.consume(len);
        let sw = Box::new(SimpleSubword{ text: blank.clone() });
        ans.default_value.subwords.push(sw);
        ans.text += &blank.clone();
    }

    fn eat_default_value(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_parameter_default_symbol();
        if num == 0 {
            return false;
        }
        ans.default_symbol = feeder.consume(num);
        ans.text += &ans.default_symbol.clone();

        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);

        while ! feeder.starts_with("}") {
            if let Some(sw) = subword::parse(feeder, core) {
                ans.text += sw.get_text();
                ans.default_value.text += sw.get_text();
                ans.default_value.subwords.push(sw);
            }

            if feeder.starts_with("\n") {
                Self::push_default_subword(1, feeder, ans);
                feeder.feed_additional_line(core);
            }

            let num = feeder.scanner_blank(core);
            if num != 0 {
                Self::push_default_subword(num, feeder, ans);
            }
        }

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

    fn eat_unknown(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if feeder.len() == 0 {
            feeder.feed_additional_line(core);
        }

        let len = feeder.scanner_unknown_in_param_brace();
        if len == 0 {
            return false;
        }

        let unknown = feeder.consume(len);
        ans.unknown += &unknown.clone();
        ans.text += &unknown;
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BracedParam> {
        if ! feeder.starts_with("${") {
            return None;
        }
        let mut ans = Self::new();
        ans.text += &feeder.consume(2);

        if Self::eat_param(feeder, &mut ans, core) {
            Self::eat_subscript(feeder, &mut ans, core);
            Self::eat_default_value(feeder, &mut ans, core);
        }

        while ! feeder.starts_with("}") {
            Self::eat_unknown(feeder, &mut ans, core);
        }

        ans.text += &feeder.consume(1);
        Some(ans)
    }
}
