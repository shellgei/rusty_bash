//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::ShellCore;
use crate::elem_subarg_command_substitution::SubArgCommandSubstitution;
use crate::elem_subarg_non_quoted::SubArgNonQuoted;
use crate::elem_subarg_double_quoted::SubArgDoubleQuoted;
use crate::elem_subarg_single_quoted::SubArgSingleQuoted;
use crate::elem_subarg_braced::SubArgBraced;
use crate::elem_subarg_variable::SubArgVariable;

pub trait ArgElem {
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<Vec<String>> {
        vec!()
    }

    fn text(&self) -> String;
    fn permit_lf(&self) -> bool {false}
}

pub fn subarg(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgVariable::parse2(text)                {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandSubstitution::parse(text, conf)    {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)            {Some(Box::new(a))}
    else if let Some(a) = SubArgBraced::parse(text, conf)        {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse(text)           {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text, conf)  {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text, conf)  {Some(Box::new(a))}
    else                                                         {None}
}

pub fn subvalue(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgVariable::parse2(text)          {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandSubstitution::parse(text, conf)    {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)      {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse2(text)    {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text, conf) {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text, conf)  {Some(Box::new(a))}
    else                                                   {None}
}

pub fn subarg_in_brace(text: &mut Feeder, conf: &mut ShellCore) -> Option<Box<dyn ArgElem>> {
    if text.len() == 0 {
        return None;
    }
    if let Some(a) = SubArgVariable::parse2(text)         {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandSubstitution::parse(text, conf)    {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)     {Some(Box::new(a))}
    else if let Some(a) = SubArgBraced::parse(text, conf)       {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text, conf) {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text, conf) {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse3(text)   {Some(Box::new(a))}
    else{None}
}

