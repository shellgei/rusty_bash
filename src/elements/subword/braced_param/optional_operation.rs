//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod replace;
mod remove;
pub mod substr;

use crate::{Feeder, ShellCore};
use crate::error::parse::ParseError;
use crate::elements::subword::Subword;
use crate::error::exec::ExecError;
use super::Param;
use self::replace::Replace;
use self::remove::Remove;
use core::fmt;
use core::fmt::Debug;

pub trait OptionalOperation {
    fn exec(&mut self, _: &Param, _: &String, _: &mut ShellCore) -> Result<String, ExecError>;
    fn boxed_clone(&self) -> Box<dyn OptionalOperation>;
    fn get_text(&self) -> String {"".to_string()}
    fn is_substr(&self) -> bool {false}
    fn is_value_check(&self) -> bool {false}
    fn set_array(&mut self, _: &Param, _: &mut Vec<String>,
                 _: &mut String, _: &mut ShellCore) -> Result<(), ExecError> {
        Ok(())
    }
    fn get_alternative(&self) -> Vec<Box<dyn Subword>> { vec![] }
}

pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Box<dyn OptionalOperation>>, ParseError> {
    if let Some(a) = Replace::parse(feeder, core)?{ Ok(Some(Box::new(a))) }
    else if let Some(a) = Remove::parse(feeder, core)?{ Ok(Some(Box::new(a))) }
//    else if let Some(a) = Substr::parse(feeder, core){ Ok(Some(Box::new(a))) }
    else{ Ok(None) }
}

impl Clone for Box::<dyn OptionalOperation> {
    fn clone(&self) -> Box<dyn OptionalOperation> {
        self.boxed_clone()
    }
}

impl Debug for dyn OptionalOperation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}
