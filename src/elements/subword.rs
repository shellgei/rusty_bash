//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod braced;
pub mod command_substitution;
pub mod double_quoted;
pub mod math_substitution;
pub mod string_double_quoted;
pub mod string_non_quoted;
pub mod single_quoted;
pub mod tilde;
pub mod variable;

use crate::{Feeder, ShellCore}; 

use self::command_substitution::SubwordCommandSubstitution;
use self::math_substitution::SubwordMathSubstitution;
use self::string_non_quoted::SubwordStringNonQuoted;
use self::double_quoted::SubwordDoubleQuoted;
use self::single_quoted::SubwordSingleQuoted;
use self::braced::SubwordBraced;
use self::variable::SubwordVariable;
use std::fmt::Debug;
use std::fmt;

pub trait Subword {
    fn eval(&mut self, _conf: &mut ShellCore, remove_lf: bool) -> Vec<Vec<String>>;
    fn get_text(&self) -> String;
    fn permit_lf(&self) -> bool {false}
}

impl Debug for dyn Subword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("SUBWORD")
            //.field("bar", &self.bar)
            //.field("baz", &self.baz)
            //.field("addr", &format_args!("{}", self.addr))
            .finish()
    }
}

pub fn parse_in_arg(text: &mut Feeder, conf: &mut ShellCore, is_in_brace: bool) -> Option<Box<dyn Subword>> {
    if let Some(a) = SubwordMathSubstitution::parse(text, conf)                   {Some(Box::new(a))}
    else if let Some(a) = SubwordCommandSubstitution::parse(text, conf)           {Some(Box::new(a))}
    else if let Some(a) = SubwordVariable::parse(text)                            {Some(Box::new(a))}
    else if let Some(a) = SubwordBraced::parse(text, conf)                        {Some(Box::new(a))}
    else if let Some(a) = SubwordSingleQuoted::parse(text, conf)                  {Some(Box::new(a))}
    else if let Some(a) = SubwordDoubleQuoted::parse(text, conf)                  {Some(Box::new(a))}
    else if let Some(a) = SubwordStringNonQuoted::parse(text, is_in_brace, false) {Some(Box::new(a))}
    else {None}
}

pub fn parse_in_value(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn Subword>> {
    if let Some(a) = SubwordMathSubstitution::parse(text, conf)               {Some(Box::new(a))}
    else if let Some(a) = SubwordCommandSubstitution::parse(text, conf)       {Some(Box::new(a))}
    else if let Some(a) = SubwordVariable::parse(text)                        {Some(Box::new(a))}
    else if let Some(a) = SubwordSingleQuoted::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = SubwordDoubleQuoted::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = SubwordStringNonQuoted::parse(text, false, true)    {Some(Box::new(a))}
    else {None}
}
