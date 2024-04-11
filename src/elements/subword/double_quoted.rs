//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{BracedParam, SimpleSubword, Subword, SubwordType};
use crate::elements::word::{Word, parameter_expansion};

#[derive(Debug, Clone)]
pub struct DoubleQuoted {
    pub text: String,
    subword_type: SubwordType,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Subword for DoubleQuoted {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn merge(&mut self, _: &Box<dyn Subword>) { panic!("SUSH INTERNAL ERROR: DoubleQuoted::merge"); }
    fn set(&mut self, _: SubwordType, _: &str) { panic!("SUSH INTERNAL ERROR: DoubleQuoted::set"); }

    fn parameter_expansion(&mut self, core: &mut ShellCore) -> bool {
        let mut word = Word::new();
        word.text = self.text.clone();
        word.subwords = self.subwords.to_vec();
        if ! parameter_expansion::eval(&mut word, core) {
            return false;
        }

        word.connect_subwords();
        self.text = "\"".to_owned() + &word.text + "\"";
        self.subwords = word.subwords;
        true
    }

    fn unquote(&mut self) {
        self.text.remove(0);
        self.text.pop();
    }

    fn get_type(&self) -> SubwordType { self.subword_type.clone()  }
    fn clear(&mut self) {
        self.text = String::new();
        self.subwords.clear();
    }
}

impl DoubleQuoted {
    pub fn new() -> DoubleQuoted {
        DoubleQuoted {
            text: String::new(),
            subword_type: SubwordType::DoubleQuoted,
            subwords: vec![],
        }
    }

    fn eat_braced_param(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(a) = BracedParam::parse(feeder, core){
            ans.text += a.get_text();
            ans.subwords.push(Box::new(a));
            true
        }else{
            false
        }
    }

    fn eat_special_or_positional_param(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_dollar_special_and_positional_param(core);
        if len == 0 {
            return false;
        }

        let txt = feeder.consume(len);
        ans.text += &txt;
        ans.subwords.push(Box::new(SimpleSubword::new(&txt, SubwordType::Parameter)));
        true
    }

    fn eat_doller(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("$") {
            ans.text += &feeder.consume(1);
            ans.subwords.push(Box::new(SimpleSubword::new("$", SubwordType::Symbol)));
            true
        }else{
            false
        }
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return false;
        }

        let txt = feeder.consume(len);
        ans.text += &txt;
        ans.subwords.push(Box::new(SimpleSubword::new(&txt, SubwordType::VarName)));
        true
    }

    fn eat_other(feeder: &mut Feeder, ans: &mut Self) -> bool {
        let len = feeder.scanner_double_quoted_subword();
        if len == 0 {
            return false;
        }

        let txt = feeder.consume(len);
        ans.text += &txt;
        ans.subwords.push(Box::new(SimpleSubword::new(&txt, SubwordType::Other)));
        true
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<DoubleQuoted> {
        if ! feeder.starts_with("\"") {
            return None;
        }
        let mut ans = Self::new();
        ans.text = feeder.consume(1);

        while Self::eat_braced_param(feeder, &mut ans, core)  
           || Self::eat_special_or_positional_param(feeder, &mut ans, core)
           || Self::eat_name(feeder, &mut ans, core)
           || Self::eat_doller(feeder, &mut ans)
           || Self::eat_other(feeder, &mut ans) {}

        if feeder.starts_with("\"") {
            ans.text += &feeder.consume(1);
            Some(ans)
        }else{
            None
        }
    }
}
