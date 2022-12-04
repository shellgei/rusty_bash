//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

//pub mod compound;

use crate::{Feeder, ShellCore}; 

use crate::elements::subarg_command_substitution::SubArgCommandSubstitution;
use crate::elements::subarg_math_substitution::SubArgMathSubstitution;
use crate::elements::subarg_string_non_quoted::SubArgStringNonQuoted;
use crate::elements::subarg_double_quoted::SubArgDoubleQuoted;
use crate::elements::subarg_single_quoted::SubArgSingleQuoted;
use crate::elements::subarg_braced::SubArgBraced;
use crate::elements::subarg_variable::SubArgVariable;

pub trait ArgElem {
    fn eval(&mut self, _conf: &mut ShellCore, as_value: bool) -> Vec<Vec<String>>;
    fn get_text(&self) -> String;
    fn permit_lf(&self) -> bool {false}
}

pub fn subarg(text: &mut Feeder, conf: &mut ShellCore, is_in_brace: bool) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgMathSubstitution::parse(text, conf)                   {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandSubstitution::parse(text, conf)           {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)                            {Some(Box::new(a))}
    else if let Some(a) = SubArgBraced::parse(text, conf)                        {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text, conf)                  {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text, conf)                  {Some(Box::new(a))}
    else if let Some(a) = SubArgStringNonQuoted::parse(text, is_in_brace, false) {Some(Box::new(a))}
    else {None}
}

pub fn subvalue(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgMathSubstitution::parse(text, conf)               {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandSubstitution::parse(text, conf)       {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)                        {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text, conf)              {Some(Box::new(a))}
    else if let Some(a) = SubArgStringNonQuoted::parse(text, false, true)    {Some(Box::new(a))}
    else {None}
}
