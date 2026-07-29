//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod remove;
mod replace;
mod substr;

use crate::{Feeder, ShellCore};
use crate::elements::parameter::Parameter;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use core::fmt;
use core::fmt::Debug;
use self::remove::Remove;
use self::replace::Replace;
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
    fn exec(&mut self, _: &Parameter, _: &str, _: &mut ShellCore) -> Result<String, ExecError>;
    fn boxed_clone(&self) -> Box<dyn BracedParamExtension>;
    fn get_text(&self) -> String;
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
-> Result<Option<Box<dyn BracedParamExtension>>, ParseError> {
    if let Some(a) = Substr::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = Remove::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = Replace::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else {
        Ok(None)
    }
}
