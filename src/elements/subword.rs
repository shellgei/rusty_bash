//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod escaped_char;
pub mod parameter;
pub mod simple;
mod single_quoted;
mod varname;

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use std::fmt;
use self::escaped_char::EscapedChar;
use self::parameter::Parameter;
use self::simple::SimpleSubword;
use self::single_quoted::SingleQuoted;
use self::varname::VarName;
use std::fmt::Debug;

impl Debug for dyn Subword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(self.get_text()).finish()
    }
}

impl Clone for Box::<dyn Subword> {
    fn clone(&self) -> Box<dyn Subword> {
        self.boxed_clone()
    }
}

fn split_str(s: &str) -> Vec<&str> {
    let mut esc = false;
    let mut from = 0;
    let mut pos = 0;
    let mut ans = vec![];

    for c in s.chars() {
        let c_len = c.len_utf8();
        pos += c_len;
        if esc || c == '\\' {
            esc = ! esc;
            continue;
        }

        if " \t\n".contains(c) {
            ans.push(&s[from..pos-c_len]);
            from = pos;
        }
    }

    ans.push(&s[from..]);
    ans
}

pub trait Subword {
    fn get_text(&self) -> &str;
    fn set_text(&mut self, _: &str) {}
    fn boxed_clone(&self) -> Box<dyn Subword>;
    fn substitute(&mut self, _: &mut ShellCore) -> Result<(), ExecError> {Ok(())}

    fn split(&self) -> Vec<Box<dyn Subword>>{
        let f = |s| Box::new( SimpleSubword {text: s}) as Box<dyn Subword>;
        split_str(self.get_text()).iter().map(|s| f(s.to_string())).collect()
    }

    fn make_glob_string(&mut self) -> String {self.get_text().to_string()}

    fn make_unquoted_string(&mut self) -> Option<String> {
        match self.get_text() {
            "" => None,
            s  => Some(s.to_string()),
        }
    }

    fn is_name(&self) -> bool {false}
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Subword>> {
    if let Some(a) = SingleQuoted::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = EscapedChar::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = Parameter::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = VarName::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = SimpleSubword::parse(feeder){ Some(Box::new(a)) }
    else{ None }
}
