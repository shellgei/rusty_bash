//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod simple;
mod braced_param;
mod command;
mod double_quoted;

use crate::{ShellCore, Feeder};
use self::simple::SimpleSubword;
use self::braced_param::BracedParam;
use self::command::CommandSubstitution;
use self::double_quoted::DoubleQuoted;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub enum SubwordType {
    /* related dollar substitution */
    BracedParameter,
    CommandSubstitution,
    Parameter,
    VarName,
    /* other subwords */
    SingleQuoted,
    DoubleQuoted,
    Symbol,
    Escaped,
    Other,
}


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
    fn boxed_clone(&self) -> Box<dyn Subword>;
    fn merge(&mut self, _right: &Box<dyn Subword>) {}
    fn set(&mut self, _subword_type: SubwordType, _s: &str) {}
    fn substitute(&mut self, core: &mut ShellCore) -> bool;

    fn split(&self, _core: &mut ShellCore) -> Vec<Box<dyn Subword>>{
        let splits = self.get_text().split('\n').collect::<Vec<&str>>();
        if splits.len() < 2 {
            return vec![self.boxed_clone()];
        }

        let mut tmp = SimpleSubword::new("", SubwordType::Other);
        let mut copy = |text| {
            tmp.set(SubwordType::Other, text);
            tmp.boxed_clone()
        };

        splits.iter().map(|s| copy(s)).collect()
    }

    fn unquote(&mut self) {}
    fn get_type(&self) -> SubwordType;
    fn clear(&mut self) {}
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Subword>> {
    if let Some(a) = BracedParam::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = CommandSubstitution::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = DoubleQuoted::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = SimpleSubword::parse(feeder, core){ Some(Box::new(a)) }
    else{ None }
}
