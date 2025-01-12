//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod alternative;
mod offset;
mod remove;
mod replace;

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;
use crate::elements::subscript::Subscript;
use crate::elements::word::Word;
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::utils;
use self::replace::Replace;
use super::simple::SimpleSubword;

#[derive(Debug, Clone, Default)]
struct Param {
    name: String,
    subscript: Option<Subscript>,
}

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    param: Param,
    replace: Option<Replace>,

    unknown: String,
    is_array: bool,
    has_alternative: bool,
    alternative_symbol: Option<String>,
    alternative_value: Option<Word>,
    num: bool,
    indirect: bool,
    has_offset: bool,
    offset: Option<ArithmeticExpr>,
    has_length: bool,
    length: Option<ArithmeticExpr>,
    has_remove_pattern: bool,
    remove_symbol: String,
    remove_pattern: Option<Word>,
    array: Vec<String>,
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str { &self.text.as_ref() }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        if ! self.check() {
            return false;
        }

        if self.indirect {
            let value = core.db.get_param(&self.param.name).unwrap_or_default();
            if utils::is_param(&value) {
                self.param.name = value;
            }else{
                eprintln!("sush: {}: invalid name", &value);
                return false;
            }
        }

        if self.param.subscript.is_some() {
            if self.param.name == "@" {
                eprintln!("sush: {}: bad substitution", &self.text);
                return false;
            }
            return self.subscript_operation(core);
        }

        if self.param.name == "@" && self.has_offset {
            return offset::set_partial_position_params(self, core);
        }

        let value = core.db.get_param(&self.param.name).unwrap_or_default();
        self.text = match self.num {
            true  => value.chars().count().to_string(),
            false => value.to_string(),
        };

        self.optional_operation(core).is_ok()
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }

    fn get_alternative_subwords(&self) -> Vec<Box<dyn Subword>> {
        match self.alternative_value.as_ref() {
            Some(w) => w.subwords.to_vec(),
            None    => vec![],
        }
    }

    fn is_array(&self) -> bool {self.is_array}
    fn get_array_elem(&self) -> Vec<String> {self.array.clone()}
}

impl BracedParam {
    fn check(&mut self) -> bool {
        if self.param.name.is_empty() || ! utils::is_param(&self.param.name) {
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

    fn subscript_operation(&mut self, core: &mut ShellCore) -> bool {
        let index = match self.param.subscript.clone().unwrap().eval(core, &self.param.name) {
            Some(s) => s,
            None => return false,
        };

        if core.db.is_assoc(&self.param.name) {
            return self.subscript_operation_assoc(core, &index);
        }

        if index.as_str() == "@" {
            self.array = core.db.get_array_all(&self.param.name);
        }

        self.text = match (self.num, index.as_str()) {
            (true, "@") => core.db.len(&self.param.name).to_string(),
            (true, _)   => core.db.get_array_elem(&self.param.name, &index).unwrap().chars().count().to_string(),
            (false, _)  => core.db.get_array_elem(&self.param.name, &index).unwrap(),
       };
       self.optional_operation(core).is_ok()
    }

    fn subscript_operation_assoc(&mut self, core: &mut ShellCore, index: &str) -> bool {
        if let Ok(s) = core.db.get_array_elem(&self.param.name, index) {
            self.text = s;
            return true;
        }
        false
    }

    fn optional_operation(&mut self, core: &mut ShellCore) -> Result<(), String> {
        if self.has_offset {
            if ! offset::set(self, core) {
                return Err("offset error".to_string());
            }
        }else if self.has_alternative {
            if ! alternative::set(self, core) {
                return Err("alternative error".to_string());
            }
        }else if self.has_remove_pattern {
            if ! remove::set(self, core) {
                return Err("remove error".to_string());
            }
        }else if let Some(r) = &self.replace {
            self.text = r.get_text(&self.text, core)?;
            /*
            if let Ok(s) = r.get_text(&self.text, core) {
                self.text = s;
                true
            }else{
                false
            }
            */
        }
        Ok(())
    }

    fn eat_subscript(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(s) = Subscript::parse(feeder, core) {
            ans.text += &s.text;
            if s.text.contains('@') {
                ans.is_array = true;
            }
            ans.param.subscript = Some(s);
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

        ans.remove_pattern = Some(Self::eat_subwords(feeder, ans, vec!["}"], core));
        true
    }

    fn eat_replace(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if ! feeder.starts_with("/") {
            return false;
        }

        let mut info = Replace::default();

        ans.text += &feeder.consume(1);
        if feeder.starts_with("/") {
            ans.text += &feeder.consume(1);
            info.all_replace = true;
        }else if feeder.starts_with("#") {
            ans.text += &feeder.consume(1);
            info.head_only_replace = true;
        }else if feeder.starts_with("%") {
            ans.text += &feeder.consume(1);
            info.tail_only_replace = true;
        }

        info.replace_from = Some(Self::eat_subwords(feeder, ans, vec!["}", "/"], core));

        if ! feeder.starts_with("/") {
            ans.replace = Some(info);
            return true;
        }
        ans.text += &feeder.consume(1);
        info.has_replace_to = true;
        info.replace_to = Some(Self::eat_subwords(feeder, ans, vec!["}"], core));

        ans.replace = Some(info);
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
        ans.alternative_value = Some(Self::eat_subwords(feeder, ans, vec!["}"], core));
        true
    }

    fn eat_subwords(feeder: &mut Feeder, ans: &mut Self, ends: Vec<&str>, core: &mut ShellCore) -> Word {
        let mut word = Word::default();
        while ! ends.iter().any(|e| feeder.starts_with(e)) {
            if let Some(sw) = subword::parse(feeder, core) {
                ans.text += sw.get_text();
                word.text += sw.get_text();
                word.subwords.push(sw);
            }else{
                let c = feeder.consume(1);
                ans.text += &c;
                word.text += &c;
                word.subwords.push(Box::new(SimpleSubword{text: c}) );
            }

            if feeder.len() == 0 {
                if ! feeder.feed_additional_line(core) {
                    return word;
                }
            }
        }

        word
    }

    fn eat_param(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if len != 0 {
            ans.param = Param{ name: feeder.consume(len), subscript: None};
            ans.text += &ans.param.name;
            return true;
        }

        let len = feeder.scanner_special_and_positional_param();
        if len != 0 {
            ans.param = Param {name: feeder.consume(len), subscript: None};
            ans.is_array = ans.param.name == "@";
            ans.text += &ans.param.name;
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
            false => {
                let len = feeder.nth(0).unwrap().len_utf8();
                feeder.consume(len)
            },
        };

        ans.unknown += &unknown.clone();
        ans.text += &unknown;
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
        }else if feeder.starts_with("!") {
            ans.indirect = true;
            ans.text += &feeder.consume(1);
        }

        if Self::eat_param(feeder, &mut ans, core) {
            Self::eat_subscript(feeder, &mut ans, core);
            let _ = Self::eat_alternative_value(feeder, &mut ans, core) 
                 || Self::eat_offset(feeder, &mut ans, core)
                 || Self::eat_remove_pattern(feeder, &mut ans, core)
                 || Self::eat_replace(feeder, &mut ans, core);
        }

        while ! feeder.starts_with("}") {
            Self::eat_unknown(feeder, &mut ans, core);
        }

        ans.text += &feeder.consume(1);
        Some(ans)
    }
}
