//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::ConditionalExpr;
use crate::elements::word::Word;
use crate::error::exec::ExecError;
use crate::ShellCore;
use std::fmt::{Display, Formatter, Result as FmtResult};

#[derive(Debug, Clone)]
pub enum CondElem {
    UnaryOp(String),
    BinaryOp(String),
    Word(Word),
    Regex(Word),
    Operand(String),
    InParen(ConditionalExpr),
    Not, // !
    And, // &&
    Or,  // ||
    Ans(bool),
}

impl CondElem {
    pub fn order(&self) -> u8 {
        match self {
            CondElem::UnaryOp(_) => 14,
            CondElem::BinaryOp(_) => 13,
            CondElem::Not => 12,
            _ => 0,
        }
    }

    pub fn eval(&mut self, core: &mut ShellCore) -> Result<(), ExecError> {
        if let CondElem::Word(ref mut w) = self {
            let new_w = w.tilde_and_dollar_expansion(core)?;
            *w = new_w;
        }
        Ok(())
    }
}

impl Display for CondElem {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let s = match self {
            CondElem::UnaryOp(op) => op.to_string(),
            CondElem::BinaryOp(op) => op.to_string(),
            CondElem::InParen(expr) => expr.text.clone(),
            CondElem::Word(w) => w.text.clone(),
            CondElem::Regex(w) => w.text.clone(),
            CondElem::Operand(op) => op.to_string(),
            CondElem::Not => "!".to_string(),
            CondElem::And => "&&".to_string(),
            CondElem::Or => "||".to_string(),
            CondElem::Ans(true) => "true".to_string(),
            CondElem::Ans(false) => "false".to_string(),
        };
        write!(f, "{s}")
    }
}
