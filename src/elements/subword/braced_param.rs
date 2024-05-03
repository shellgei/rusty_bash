//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct BracedParam {
    pub text: String,
    pub name: String,
    pub subscript: String,
    subword_type: SubwordType,
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
        if self.name.len() == 0 {
            return false;
        }

        if ! is_param(&self.name) {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }

        let value = core.data.get_param_ref(&self.name);
        self.text = value.to_string();
        true
    }

    fn get_type(&self) -> SubwordType { self.subword_type.clone()  }
    fn clear(&mut self) { self.text = String::new(); }
}

impl BracedParam {
    fn new() -> BracedParam {
        BracedParam {
            text: String::new(),
            name: String::new(),
            subscript: String::new(),
            subword_type: SubwordType::BracedParameter,
        }
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

    fn eat(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if feeder.len() == 0 {
            feeder.feed_additional_line(core);
        }

        match subword::parse(feeder, core) {
            Some(sw) => {
                ans.text += sw.get_text();
                return sw.get_text() != "}"; //end if "}"
            },
            None => {
                match feeder.scanner_unknown_in_param_brace() {
                    0 => {
                        ans.text.clear();
                        return false;
                    },
                    len => {
                        ans.text += &feeder.consume(len);
                        return true;
                    },
                }
            },
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BracedParam> {
        if ! feeder.starts_with("${") {
            return None;
        }
        let mut ans = Self::new();
        ans.text += &feeder.consume(2);

        let mut num = 0;
        while ! feeder.starts_with("}") {
            if ! Self::eat_param(feeder, &mut ans, core) {
                Self::eat(feeder, &mut ans, core);
            }
            num += 1;
        }

        if num > 1 {
            ans.name.clear();
        }

        if feeder.starts_with("}") {
            ans.text += &feeder.consume(1);
            Some(ans)
        }else{
            feeder.consume(feeder.len());
            None
        }
    }
}
