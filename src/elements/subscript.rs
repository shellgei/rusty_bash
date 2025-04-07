//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::error::exec::ExecError;
use crate::error::parse::ParseError;
use super::expr::arithmetic::ArithmeticExpr;

#[derive(Debug, Clone, Default)]
pub struct Subscript {
    pub text: String,
    pub inner: Option<ArithmeticExpr>,
    pub inner_special: String,
}

impl Subscript {
    pub fn eval(&mut self, core: &mut ShellCore, param_name: &str) -> Result<String, ExecError> {
        if self.inner_special != "" {
            return Ok(self.inner_special.clone());
        }

        if let Some(a) = self.inner.as_mut() {
            if a.text.chars().all(|c| " \t\n".contains(c)) {
                return Err(ExecError::ArrayIndexInvalid(a.text.clone()));
            }
            return match core.db.is_assoc(param_name) {
                true  => {
                    dbg!("{:?}", &self.inner);
                    match self.inner.as_mut() {
                        Some(sub) => sub.eval_as_assoc_index(core),
                        None => Err(ExecError::ArrayIndexInvalid("".to_string())),
                    }
                },
                false => a.eval(core),
            };
        }

        Err(ExecError::ArrayIndexInvalid("".to_string()))
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Result<Option<Self>, ParseError> {
        if ! feeder.starts_with("[") {
            return Ok(None);
        }

        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        if feeder.starts_with("@") {
            ans.text += "@";
            ans.inner_special = feeder.consume(1);
        }else if feeder.starts_with("*") {
            ans.text += "*";
            ans.inner_special = feeder.consume(1);
        }else if let Some(a) = ArithmeticExpr::parse(feeder, core, true, "[")? {
            ans.text += &a.text.clone();
            ans.inner = Some(a);
        }

        if ! feeder.starts_with("]") {
            return Ok(None);
        }

        ans.text += &feeder.consume(1);
        Ok(Some(ans))
    }
}
