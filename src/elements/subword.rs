//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod simple;

use crate::{Feeder, ShellCore};
use std::fmt;
use self::simple::SimpleSubword;
use std::fmt::Debug;

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
    fn set_text(&mut self, _: &str) {}
    fn boxed_clone(&self) -> Box<dyn Subword>;
}

pub fn parse(feeder: &mut Feeder, _: &mut ShellCore) -> Option<Box<dyn Subword>> {
    if let Some(a) = SimpleSubword::parse(feeder){ Some(Box::new(a)) }
    else{ None }
}
