//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::ConditionalExpr;
use crate::elements::word::Word;
use crate::error::exec::ExecError;
use crate::ShellCore;
use std::fmt;

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
        if let CondElem::Word(w) = self {
            let new_w = w.tilde_and_dollar_expansion(core)?;
            *w = new_w;
        }
        Ok(())
    }
}

impl fmt::Display for CondElem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CondElem::UnaryOp(op) | CondElem::BinaryOp(op) | CondElem::Operand(op) => {
                write!(f, "{op}")
            }
            CondElem::InParen(expr) => write!(f, "{}", expr.text),
            CondElem::Word(w) | CondElem::Regex(w) => write!(f, "{}", w.text),
            CondElem::Not => write!(f, "!"),
            CondElem::And => write!(f, "&&"),
            CondElem::Or => write!(f, "||"),
            CondElem::Ans(true) => write!(f, "true"),
            CondElem::Ans(false) => write!(f, "false"),
        }
    }
}
