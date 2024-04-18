//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::word::{Word, substitution};
use crate::elements::subword::CommandSubstitution;
use super::{BracedParam, SimpleSubword, Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct DoubleQuoted {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
}

impl Subword for DoubleQuoted {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        let mut word = Word::new();
        word.subwords = self.subwords.to_vec();
        if ! substitution::eval(&mut word, core) {
            return false;
        }
        self.subwords = word.subwords;
        self.text = self.subwords.iter().map(|s| s.get_text()).collect();
        self.text = "\"".to_owned() + &self.text + "\"";
        true
    }

    fn unquote(&mut self) {
        self.subwords.iter_mut().for_each(|sw| sw.unquote());
        self.text = self.subwords.iter().map(|s| s.get_text()).collect();
    }

    fn get_type(&self) -> SubwordType { SubwordType::DoubleQuoted  }
}

impl DoubleQuoted {
    pub fn new() -> DoubleQuoted {
        DoubleQuoted {
            text: String::new(),
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

    fn eat_command_substitution(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(a) = CommandSubstitution::parse(feeder, core){
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
        match feeder.starts_with("$") {
            true  => Self::set_subword(feeder, ans, 1, SubwordType::Symbol),
            false => false,
        }
    }

    fn eat_escaped_char(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if feeder.starts_with("\\$") || feeder.starts_with("\\\\") {
            return Self::set_subword(feeder, ans, 2, SubwordType::Escaped);
        }
        let len = feeder.scanner_escaped_char(core);
        Self::set_subword(feeder, ans, len, SubwordType::Other)
    }

    fn eat_name(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        Self::set_subword(feeder, ans, len, SubwordType::VarName)
    }

    fn eat_other(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_double_quoted_subword(core);
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
               || Self::eat_command_substitution(feeder, &mut ans, core)
               || Self::eat_special_or_positional_param(feeder, &mut ans, core)
               || Self::eat_doller(feeder, &mut ans)
               || Self::eat_escaped_char(feeder, &mut ans, core)
               || Self::eat_name(feeder, &mut ans, core)
               || Self::eat_other(feeder, &mut ans, core) {}

            if feeder.starts_with("\"") {
                ans.text += &feeder.consume(1);
//                eprintln!("{:?}", &ans);
                return Some(ans);
            }else if feeder.len() > 0 {
                panic!("SUSH INTERNAL ERROR: unknown chars in double quoted word");
            }else if ! feeder.feed_additional_line(core) {
                return None;
            }
        }
    }
}
