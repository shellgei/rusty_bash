//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword;
use crate::elements::subword::Subword;
use crate::elements::subscript::Subscript;
use crate::elements::word::Word;

#[derive(Debug, Clone)]
pub struct BracedParam {
    pub text: String,
    pub name: String,
    pub subscript: Option<Subscript>,
    pub default_symbol: String,
    pub default_value: Word,
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

    fn substitute(&mut self, core: &mut ShellCore) -> bool {
        if self.name.len() == 0 || ! is_param(&self.name) {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }

        if let Some(sub) = self.subscript.as_mut() {
            if let Some(s) = sub.eval() {
                self.text = core.data.get_array(&self.name, &s);
            }
        }else{
            let value = core.data.get_param(&self.name);
            self.text = value.to_string();
        }

        if self.text == "" {
            self.text = self.replace_to_default(core);
        }

        true
    }

    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }
}

impl BracedParam {
    fn new() -> BracedParam {
        BracedParam {
            text: String::new(),
            name: String::new(),
            subscript: None,
            default_value: Word::new(),
            default_symbol: String::new(),
        }
    }

    fn replace_to_default(&mut self, core: &mut ShellCore) -> String {
        if self.default_symbol == ":-" {
            if let Some(s) = self.default_value.eval_as_value(core) {
                return s;
            }
            return String::new();
        }

        if self.default_symbol == ":=" {
            if let Some(s) = self.default_value.eval_as_value(core) {
                core.data.set_param(&self.name, &s);
                return s;
            }
            return String::new();
        }

        String::new()
    }

    fn eat_subscript(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        if let Some(s) = Subscript::parse(feeder, core) {
            ans.text += &s.text;
            ans.subscript = Some(s);
            return true;
        }

        false
    }

    fn eat_default_value(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_parameter_checker();
        if num == 0 {
            return false;
        }
        ans.default_symbol = feeder.consume(num);
        ans.text += &ans.default_symbol.clone();

        loop {
            match subword::parse(feeder, core) {
                Some(sw) => {
                    if sw.get_text() == "}" {
                        return true;
                    }

                    ans.text += sw.get_text();
                    ans.default_value.text += sw.get_text();
                    ans.default_value.subwords.push(sw);
                },
                _ => {},
            }
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

    fn eat_unknown(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
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
        let mut default_exists = false;
        while ! feeder.starts_with("}") {
            if Self::eat_param(feeder, &mut ans, core) {
                Self::eat_subscript(feeder, &mut ans, core);
                default_exists = Self::eat_default_value(feeder, &mut ans, core);

                if default_exists {
                    break;
                }
            
            }else{
                Self::eat_unknown(feeder, &mut ans, core);
            }
            num += 1;
        }

        if num > 1 {
            ans.name.clear();
        }

        if default_exists {
            Some(ans)
        }else if feeder.starts_with("}") {
            ans.text += &feeder.consume(1);
            Some(ans)
        }else{
            feeder.consume(feeder.len());
            None
        }
    }
}
