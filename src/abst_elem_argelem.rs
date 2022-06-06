//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::ShellCore;
use crate::elems_in_arg::{SubArgNonQuoted, SubArgBraced, SubArgSingleQuoted, SubArgDoubleQuoted, SubArgVariable, SubArgCommandExp};

pub trait ArgElem {
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> {
        vec!()
    }

    fn text(&self) -> String;
}

pub fn subarg(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgVariable::parse2(text)          {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandExp::parse(text)    {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)      {Some(Box::new(a))}
    else if let Some(a) = SubArgBraced::parse(text)        {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse(text)     {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text) {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text)  {Some(Box::new(a))}
    else                                                   {None}
}

pub fn subvalue(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgVariable::parse2(text)          {Some(Box::new(a))}
    else if let Some(a) = SubArgCommandExp::parse(text)    {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)      {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse2(text)    {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text) {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text)  {Some(Box::new(a))}
    else                                                   {None}
}

pub fn subarg_in_brace(text: &mut Feeder) -> Option<Box<dyn ArgElem>> {
    if let Some(a) = SubArgVariable::parse2(text)         {Some(Box::new(a))}
    else if let Some(a) = SubArgVariable::parse(text)     {Some(Box::new(a))}
    else if let Some(a) = SubArgBraced::parse(text)       {Some(Box::new(a))}
    else if let Some(a) = SubArgSingleQuoted::parse(text) {Some(Box::new(a))}
    else if let Some(a) = SubArgDoubleQuoted::parse(text) {Some(Box::new(a))}
    else if let Some(a) = SubArgNonQuoted::parse3(text)   {Some(Box::new(a))}
    else{None}
}

