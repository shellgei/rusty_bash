//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

//pub mod compound;

use crate::{Feeder, ShellCore}; 

use crate::elements::subword_command_substitution::SubWordCommandSubstitution;
use crate::elements::subword_math_substitution::SubWordMathSubstitution;
use crate::elements::subword_string_non_quoted::SubWordStringNonQuoted;
use crate::elements::subword_double_quoted::SubWordDoubleQuoted;
use crate::elements::subword_single_quoted::SubWordSingleQuoted;
use crate::elements::subword_braced::SubWordBraced;
use crate::elements::subword_variable::SubWordVariable;

pub trait WordElem {
    fn eval(&mut self, _conf: &mut ShellCore, as_value: bool) -> Vec<Vec<String>>;
    fn get_text(&self) -> String;
    fn permit_lf(&self) -> bool {false}
}

pub fn parse_in_word(text: &mut Feeder, conf: &mut ShellCore, is_in_brace: bool) -> Option<Box<dyn WordElem>> {
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
