//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{SimpleSubword, Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct DoubleQuoted {
    pub text: String,
    subword_type: SubwordType,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Subword for DoubleQuoted {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn merge(&mut self, right: &Box<dyn Subword>) { }
    fn set(&mut self, subword_type: SubwordType, s: &str){ }
    fn parameter_expansion(&mut self, core: &mut ShellCore) -> bool {false}
    fn unquote(&mut self) { }
    fn get_type(&self) -> SubwordType { self.subword_type.clone()  }
    fn clear(&mut self) { self.text = String::new(); }
}

impl DoubleQuoted {
    pub fn new() -> DoubleQuoted {
        DoubleQuoted {
            text: String::new(),
            subword_type: SubwordType::DoubleQuoted,
            subwords: vec![],
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<DoubleQuoted> {
        if ! feeder.starts_with("\"") {
            return None;
        }
        let mut ans = Self::new();
        ans.text = feeder.consume(1);

        let len = feeder.scanner_double_quoted_subword();
        if len > 0 {
            let txt = feeder.consume(len);
            ans.text += &txt;
            ans.subwords.push(Box::new(SimpleSubword::new(&txt, SubwordType::Other)));
        }

        if feeder.starts_with("\"") {
            ans.text += &feeder.consume(1);
            Some(ans)
        }else{
            None
        }
    }
}
