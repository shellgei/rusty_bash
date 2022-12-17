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

use crate::element::subword::command_substitution::SubWordCommandSubstitution;
use crate::element::subword::math_substitution::SubWordMathSubstitution;
use crate::element::subword::string_non_quoted::SubWordStringNonQuoted;
use crate::element::subword::double_quoted::SubWordDoubleQuoted;
use crate::element::subword::single_quoted::SubWordSingleQuoted;
use crate::element::subword::braced::SubWordBraced;
use crate::element::subword::variable::SubWordVariable;

pub trait WordElem {
    fn eval(&mut self, _conf: &mut ShellCore, remove_lf: bool) -> Vec<Vec<String>>;
    fn get_text(&self) -> String;
    fn permit_lf(&self) -> bool {false}
}

pub fn parse_in_arg(text: &mut Feeder, conf: &mut ShellCore, is_in_brace: bool) -> Option<Box<dyn WordElem>> {
    if let Some(a) = SubWordMathSubstitution::parse(text, conf)                   {Some(Box::new(a))}
    else if let Some(a) = SubWordCommandSubstitution::parse(text, conf)           {Some(Box::new(a))}
    else if let Some(a) = SubWordVariable::parse(text)                            {Some(Box::new(a))}
    else if let Some(a) = SubWordBraced::parse(text, conf)                        {Some(Box::new(a))}
    else if let Some(a) = SubWordSingleQuoted::parse(text, conf)                  {Some(Box::new(a))}
    else if let Some(a) = SubWordDoubleQuoted::parse(text, conf)                  {Some(Box::new(a))}
    else if let Some(a) = SubWordStringNonQuoted::parse(text, is_in_brace, false) {Some(Box::new(a))}
    else {None}
}

pub fn parse_in_value(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn WordElem>> {
    if let Some(a) = SubWordMathSubstitution::parse(text, conf)               {Some(Box::new(a))}
    else if let Some(a) = SubWordCommandSubstitution::parse(text, conf)       {Some(Box::new(a))}
    else if let Some(a) = SubWordVariable::parse(text)                        {Some(Box::new(a))}
    else if let Some(a) = SubWordSingleQuoted::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = SubWordDoubleQuoted::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = SubWordStringNonQuoted::parse(text, false, true)    {Some(Box::new(a))}
    else {None}
}
