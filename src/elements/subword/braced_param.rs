//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct BracedParam {
    pub text: String,
    subword_type: SubwordType,
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn merge(&mut self, right: &Box<dyn Subword>) {
        self.text += &right.get_text();
    }

    fn set(&mut self, subword_type: SubwordType, s: &str){
        self.text = s.to_string();
        self.subword_type = subword_type;
    }

    fn parameter_expansion(&mut self, core: &mut ShellCore) {
        let len = self.text.len();
        let value = core.get_param_ref(&self.text[2..len-1]);
        self.text = value.to_string();
    }

    fn get_type(&self) -> SubwordType { self.subword_type.clone()  }
    fn clear(&mut self) { self.text = String::new(); }
}

impl BracedParam {
    fn new() -> BracedParam {
        BracedParam {
            text: String::new(),
            subword_type: SubwordType::BracedParameter,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BracedParam> {
        if ! feeder.starts_with("${") {
            return None;
        }
        let mut ans = Self::new();
        ans.text += &feeder.consume(2);

        loop {
            let len = feeder.scanner_blank(core);
            ans.text += &feeder.consume(len);
            match subword::parse(feeder, core) {
                Some(sw) => {
                    ans.text += sw.get_text();
                    if sw.get_text() == "}" {
                        return Some(ans);
                    }
                },
                _ => {
                    let len = feeder.scanner_unknown_in_param_brace(core);
                    ans.text += &feeder.consume(len);
                    
                    if feeder.starts_with("}") {
                        ans.text += &feeder.consume(1);
                        return Some(ans);
                    }

                    if ! feeder.feed_additional_line(core) {
                        return None;
                    }
                },
            }
        }
    }
}
