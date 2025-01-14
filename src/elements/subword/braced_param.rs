//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod value_check;
mod substr;
mod remove;
mod replace;

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;
use crate::elements::subscript::Subscript;
use crate::elements::word::Word;
use crate::utils;
use crate::utils::error::ParseError;
use self::remove::Remove;
use self::replace::Replace;
use self::substr::Substr;
use self::value_check::ValueCheck;
use super::simple::SimpleSubword;

#[derive(Debug, Clone, Default)]
struct Param {
    name: String,
    subscript: Option<Subscript>,
}

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    array: Vec<String>,

    param: Param,
    replace: Option<Replace>,
    substr: Option<Substr>,
    remove: Option<Remove>,
    value_check: Option<ValueCheck>,

    unknown: String,
    is_array: bool,
    num: bool,
    indirect: bool,
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str { &self.text.as_ref() }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> Result<(), String> {
        if ! self.check() {
            return Err(format!("{}: bad substitution", &self.text));
        }

        if self.indirect {
            let value = core.db.get_param(&self.param.name).unwrap_or_default();
            if utils::is_param(&value) {
                self.param.name = value;
            }else{
                return Err(format!("sush: {}: invalid name", &value));
            }
        }

        if self.param.subscript.is_some() {
            if self.param.name == "@" {
                return Err(format!("sush: @: bad substitution"));
            }
            return self.subscript_operation(core);
        }

        if self.param.name == "@" {
            if let Some(s) = self.substr.as_mut() {
                return s.set_partial_position_params(&mut self.array, &mut self.text, core);
            }
        }

        let value = core.db.get_param(&self.param.name).unwrap_or_default();
        self.text = match self.num {
            true  => value.chars().count().to_string(),
            false => value.to_string(),
        };

        self.optional_operation(core)
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }

    fn get_alternative_subwords(&self) -> Vec<Box<dyn Subword>> {
        if self.value_check.is_none() {
            return vec![];
        }

        let check = self.value_check.clone().unwrap();
        match &check.alternative_value {
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

    fn subscript_operation(&mut self, core: &mut ShellCore) -> Result<(), String> {
        let index = self.param.subscript.clone().unwrap().eval(core, &self.param.name)?;
        /*
        let index = match self.param.subscript.clone().unwrap().eval(core, &self.param.name) {
            Some(s) => s,
            None => return Err("index evaluation error".to_string()),
        };*/

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
       self.optional_operation(core)
    }

    fn subscript_operation_assoc(&mut self, core: &mut ShellCore, index: &str) -> Result<(), String> {
        let s = core.db.get_array_elem(&self.param.name, index)?;
        self.text = s;
        Ok(())
    }

    fn optional_operation(&mut self, core: &mut ShellCore) -> Result<(), String> {
        self.text = if let Some(s) = self.substr.as_mut() {
            s.get_text(&self.text, core)?
        }else if let Some(v) = self.value_check.as_mut() {
            v.set(&self.param.name, &self.text, core)?
        }else if let Some(r) = self.remove.as_mut() {
            r.set(&mut self.text, core)?
        }else if let Some(r) = &self.replace {
            r.get_text(&self.text, core)?
        }else{
            self.text.clone()
        };

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

    fn eat_subwords(feeder: &mut Feeder, ans: &mut Self, ends: Vec<&str>, core: &mut ShellCore) -> Word {
        let mut word = Word::default();
        while ! ends.iter().any(|e| feeder.starts_with(e)) {
            if let Ok(Some(sw)) = subword::parse(feeder, core) {
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
                if ! feeder.feed_additional_line(core).is_ok() {
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
            let _ = feeder.feed_additional_line(core);
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

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("${") {
            return Ok(None);
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
            let _ = ValueCheck::eat(feeder, &mut ans, core) 
                 || Substr::eat(feeder, &mut ans, core)
                 || Remove::eat(feeder, &mut ans, core)
                 || Replace::eat(feeder, &mut ans, core);
        }

        while ! feeder.starts_with("}") {
            Self::eat_unknown(feeder, &mut ans, core);
        }

        ans.text += &feeder.consume(1);
        Ok(Some(ans))
    }
}
