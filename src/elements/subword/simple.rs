//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct SimpleSubword {
    pub text: String,
    subword_type: SubwordType,
}

impl Subword for SimpleSubword {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn merge(&mut self, right: &Box<dyn Subword>) {
        self.text += &right.get_text();
    }

    fn set(&mut self, subword_type: SubwordType, s: &str){
        self.text = s.to_string();
        self.subword_type = subword_type;
    }

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        match self.subword_type {
            SubwordType::Parameter => {
                let value = core.data.get_param(&self.text[1..]);
                self.text = value.to_string();
            },
            _ => {},
        }
        true
    }

    fn quote_to_escape(&mut self) {
        if ! self.text.starts_with("'") 
        || ! self.text.ends_with("'") {
            return;
        }
        self.text.pop();
        self.text.remove(0);

        self.text = self.text
            .replace("\\", "\\\\")
            .replace("*", "\\*")
            .replace("?", "\\?")
            .replace("[", "\\[")
            .replace("]", "\\]");

        self.subword_type = SubwordType::ConvertedQuoted;
    }

    fn unquote(&mut self) {
        match self.subword_type {
            SubwordType::ConvertedQuoted => {
                let mut ans = vec![];
                let mut quoted = false;
                for c in self.text.chars() {
                    if quoted {
                        quoted = false;
                        ans.push(c);
                    }else if c == '\\' {
                        quoted = true;
                    }else {
                        ans.push(c);
                    }
                }

                self.text = ans.iter().collect();
            },
            SubwordType::Escaped => {
                self.text.remove(0);
            },
            _ => {},
        }
    }

    fn unquote2(&mut self) {
        match self.subword_type {
            SubwordType::SingleQuoted => {
                self.text.remove(0);
                self.text.pop();
            },
            _ => {},
        }
    }

    fn get_type(&self) -> SubwordType { self.subword_type.clone()  }
    fn clear(&mut self) { self.text = String::new(); }
}

impl SimpleSubword {
    pub fn new(s: &str, tp: SubwordType) -> SimpleSubword {
        SimpleSubword {
            text: s.to_string(),
            subword_type: tp,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SimpleSubword> {
        let len = feeder.scanner_dollar_special_and_positional_param(core);
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::Parameter));
        }

        let len = feeder.scanner_name(core);
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::VarName));
        }

        let len = feeder.scanner_single_quoted_subword(core);
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::SingleQuoted));
        }
    
        let len = feeder.scanner_escaped_char(core);
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::Escaped));
        }

        let len = feeder.scanner_subword_symbol();
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::Symbol));
        }

        let len = feeder.scanner_subword();
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::Other));
        }

        None
    }
}
