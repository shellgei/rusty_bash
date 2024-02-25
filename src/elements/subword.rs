//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod simple;

use crate::{ShellCore, Feeder};
use crate::elements::subword::simple::SimpleSubword;
use std::fmt;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub enum SubwordType {
    /* parameters and variables */
    Parameter,
    VarName,
    /* simple subwords */
    SingleQuoted,
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
    fn merge(&mut self, right: &Box<dyn Subword>);
    fn set(&mut self, subword_type: SubwordType, s: &str);
    fn parameter_expansion(&mut self, core: &mut ShellCore);
    fn unquote(&mut self);
    fn get_type(&self) -> SubwordType;
    fn clear(&mut self);
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Subword>> {
    if let Some(a) = SimpleSubword::parse(feeder, core){ Some(Box::new(a)) }
    else{ None }
}
