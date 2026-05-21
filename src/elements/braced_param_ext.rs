//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod substr;

use crate::{Feeder, ShellCore};
use crate::error::parse::ParseError;
use core::fmt;
use core::fmt::Debug;
use self::substr::Substr;

impl Clone for Box<dyn BracedExcludeension> {
    fn clone(&self) -> Box<dyn BracedExcludeension> {
        self.boxed_clone()
    }
}

impl Debug for dyn BracedExcludeension {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}

pub trait BracedExcludeension {
    fn boxed_clone(&self) -> Box<dyn BracedExcludeension>;
    fn get_text(&self) -> String;
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
-> Result<Option<Box<dyn BracedExcludeension>>, ParseError> {
    if let Some(a) = Substr::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else {
        Ok(None)
    }
}
