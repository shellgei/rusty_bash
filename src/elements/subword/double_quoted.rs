//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::{SimpleSubword, Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct DoubleQuoted {
    text: String,
    subwords: Vec<Box<dyn Subword>>,
}

impl Subword for DoubleQuoted {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn parameter_expansion(&mut self, core: &mut ShellCore) -> bool {true}
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
            while Self::eat_other(feeder, &mut ans, core) {}

            if feeder.starts_with("\"") {
                ans.text += &feeder.consume(1);
                eprintln!("{:?}", &ans);
                return Some(ans);
            }else if feeder.len() > 0 {
                panic!("SUSH INTERNAL ERROR: unknown chars in double quoted word");
            }else if ! feeder.feed_additional_line(core) {
                return None;
            }
        }
    }
}
