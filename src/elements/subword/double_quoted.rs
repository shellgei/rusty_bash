//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{BracedParam, SimpleSubword, Subword, SubwordType};
use crate::elements::word::{parameter_expansion, Word};

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
        word.subwords = self.subwords.to_vec();
        if ! parameter_expansion::eval(&mut word, core) {
            return false;
        }
        self.text = self.subwords.iter().map(|s| s.get_text()).collect();
        self.text.insert(0, '"');
        self.text.push_str("\"");
        self.subwords = word.subwords;
        true
    }

    fn unquote(&mut self) {
        self.subwords.iter_mut().for_each(|sw| sw.unquote());
        self.text = self.subwords.iter().map(|s| s.get_text()).collect();
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

    fn set_subword(feeder: &mut Feeder, ans: &mut Self, len: usize, tp: SubwordType) -> bool {
        if len == 0 {
            return false;
        }

        let txt = feeder.consume(len);
        ans.text += &txt;
        ans.subwords.push(Box::new(SimpleSubword::new(&txt, tp)));
        true
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
        Self::set_subword(feeder, ans, len, SubwordType::Parameter)
    }

    fn eat_doller(feeder: &mut Feeder, ans: &mut Self) -> bool {
        if feeder.starts_with("$") {
            Self::set_subword(feeder, ans, 1, SubwordType::Symbol)
        }else{
            false
        }
    }

    fn eat_escaped_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_escaped_char(core);
        if len < 2 {
            return false;
        }
        
        match feeder.nth(1).unwrap() {
            '$' | '\\' => Self::set_subword(feeder, ans, len, SubwordType::Escaped),
            _          => Self::set_subword(feeder, ans, len, SubwordType::Other),
        }
    }

    fn eat_name_or_other(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if Self::set_subword(feeder, ans, len, SubwordType::VarName) {
            return true;
        }

        let len = feeder.scanner_double_quoted_subword();
        Self::set_subword(feeder, ans, len, SubwordType::Other) 
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<DoubleQuoted> {
        if ! feeder.starts_with("\"") {
            return None;
        }
        let mut ans = Self::new();
        ans.text = feeder.consume(1);

        loop {
            while Self::eat_braced_param(feeder, &mut ans, core)  
               || Self::eat_special_or_positional_param(feeder, &mut ans, core)
               || Self::eat_doller(feeder, &mut ans)
               || Self::eat_escaped_char(feeder, &mut ans, core)
               || Self::eat_name_or_other(feeder, &mut ans, core) {}
    
            if feeder.starts_with("\"") {
                ans.text += &feeder.consume(1);
                return Some(ans);
            }else if ! feeder.feed_additional_line(core) {
                return None;
            }
        }
    }
}
