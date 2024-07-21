//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod escaped_char;
mod parameter;
mod simple;
mod single_quoted;

use crate::{Feeder, ShellCore};
use std::fmt;
use self::escaped_char::EscapedChar;
use self::parameter::Parameter;
use self::simple::SimpleSubword;
use self::single_quoted::SingleQuoted;
use std::fmt::Debug;

impl Debug for dyn Subword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}

impl Clone for Box::<dyn Subword> {
    fn clone(&self) -> Box<dyn Subword> {
        self.boxed_clone()
    }
}

pub trait Subword {
    fn get_text(&self) -> &str;
    fn set_text(&mut self, _: &str) {}
    fn boxed_clone(&self) -> Box<dyn Subword>;
    fn substitute(&mut self, _: &mut ShellCore) -> bool {true}

    fn make_unquoted_string(&mut self) -> Option<String> {
        match self.get_text() {
            "" => None,
            s  => Some(s.to_string()),
        }
    }
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Subword>> {
    if let Some(a) = SingleQuoted::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = EscapedChar::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = Parameter::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = SimpleSubword::parse(feeder){ Some(Box::new(a)) }
    else{ None }
}
