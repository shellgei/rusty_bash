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

use crate::element::subword::command_substitution::SubwordCommandSubstitution;
use crate::element::subword::math_substitution::SubwordMathSubstitution;
use crate::element::subword::string_non_quoted::SubwordStringNonQuoted;
use crate::element::subword::double_quoted::SubwordDoubleQuoted;
use crate::element::subword::single_quoted::SubwordSingleQuoted;
use crate::element::subword::braced::SubwordBraced;
use crate::element::subword::variable::SubwordVariable;

pub trait Subword {
    fn eval(&mut self, _conf: &mut ShellCore, remove_lf: bool) -> Vec<Vec<String>>;
    fn get_text(&self) -> String;
    fn permit_lf(&self) -> bool {false}
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
