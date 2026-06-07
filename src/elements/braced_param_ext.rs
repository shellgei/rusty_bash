//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod substr;

use crate::{Feeder, ShellCore};
use crate::elements::substitution::variable::Variable;
use crate::error::parse::ParseError;
use crate::error::exec::ExecError;
use core::fmt;
use core::fmt::Debug;
use self::substr::Substr;

impl Clone for Box<dyn BracedParamExtension> {
    fn clone(&self) -> Box<dyn BracedParamExtension> {
        self.boxed_clone()
    }
}

impl Debug for dyn BracedParamExtension {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}

pub trait BracedParamExtension {
    fn exec(&mut self, _: &Variable, _: &str, _: &mut ShellCore) -> Result<String, ExecError>;
    fn boxed_clone(&self) -> Box<dyn BracedParamExtension>;
    fn get_text(&self) -> String;
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
-> Result<Option<Box<dyn BracedParamExtension>>, ParseError> {
    if let Some(a) = Substr::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else {
        Ok(None)
    }
}
