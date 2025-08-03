//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

mod case_conv;
mod remove;
mod replace;
mod substr;
mod value_check;

use self::case_conv::CaseConv;
use self::remove::Remove;
use self::replace::Replace;
use self::substr::Substr;
use self::value_check::ValueCheck;
use super::Variable;
use crate::elements::subword::Subword;
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};
use core::fmt;
use core::fmt::Debug;

pub trait OptionalOperation {
    fn exec(&mut self, _: &Variable, _: &str, _: &mut ShellCore) -> Result<String, ExecError>;
    fn boxed_clone(&self) -> Box<dyn OptionalOperation>;
    fn get_text(&self) -> String;
    fn has_array_replace(&self) -> bool {
        false
    }
    fn is_value_check(&self) -> bool {
        false
    }
    fn set_array(
        &mut self,
        _: &Variable,
        _: &mut Vec<String>,
        _: &mut String,
        _: &mut ShellCore,
    ) -> Result<(), ExecError> {
        Ok(())
    }
    fn get_alternative(&self) -> Vec<Box<dyn Subword>> {
        vec![]
    }

    fn set_heredoc_flag(&mut self) {}
}

pub fn parse(
    feeder: &mut Feeder,
    core: &mut ShellCore,
) -> Result<Option<Box<dyn OptionalOperation>>, ParseError> {
    if let Some(a) = Replace::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = ValueCheck::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = CaseConv::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = Remove::parse(feeder, core)? {
        Ok(Some(Box::new(a)))
    } else if let Some(a) = Substr::parse(feeder, core) {
        Ok(Some(Box::new(a)))
    } else {
        Ok(None)
    }
}

impl Clone for Box<dyn OptionalOperation> {
    fn clone(&self) -> Box<dyn OptionalOperation> {
        self.boxed_clone()
    }
}

impl Debug for dyn OptionalOperation {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(&self.get_text()).finish()
    }
}
