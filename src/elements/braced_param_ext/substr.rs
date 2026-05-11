//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use super::BracedParamExtension;

/*
use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::elements::substitution::variable::Variable;
use crate::error::arith::ArithError;
use crate::error::exec::ExecError;
use crate::{Feeder, ShellCore};
*/

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: Option<Word>,
    pub length: Option<Word>,
}

impl BracedParamExtension for Substr {
    fn get_text(&self) -> String { self.text.clone() }
    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> { Box::new(self.clone()) }
}
