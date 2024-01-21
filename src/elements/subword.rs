//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

pub mod brace;
pub mod unquoted;

use crate::{Feeder, ShellCore};
use crate::elements::subword::brace::BraceSubword;
use super::subword::unquoted::UnquotedSubword;
use std::fmt;
use std::fmt::Debug;

impl Debug for dyn Subword {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}

pub trait Subword {
    fn get_text(&self) -> String;
    fn eval(&mut self) -> Vec<Vec<String>>;
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Box<dyn Subword>> {
    if let Some(a)      = BraceSubword::parse(feeder, core){ Some(Box::new(a)) }
    else if let Some(a) = UnquotedSubword::parse(feeder, core){ Some(Box::new(a)) }
    else{ None }
}
