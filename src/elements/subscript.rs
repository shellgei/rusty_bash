//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::expr::arithmetic::ArithmeticExpr;

#[derive(Debug, Clone, Default)]
pub struct Subscript {
    pub text: String,
    pub inner: Option<ArithmeticExpr>,
    pub inner_special: String,
}

impl Subscript {
    pub fn eval(&mut self, core: &mut ShellCore, param_name: &str) -> Result<String, String> {
        if self.inner_special != "" {
            return Ok(self.inner_special.clone());
        }

        if let Some(a) = self.inner.as_mut() {
            if a.text.chars().all(|c| " \t\n".contains(c)) {
                return Err("invalid inner".to_string());
            }
            return match core.db.is_assoc(param_name) {
                true  => {
                    match self.inner.as_mut() {
                        Some(sub) => sub.eval_as_assoc_index(core),
                        None => Err("no inner".to_string()),
                    }
                },
                false => a.eval(core),
            };
        }

        Err("evaluation failure".to_string())
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("[") {
            return None;
        }

        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        if feeder.starts_with("@") {
            ans.text += "@";
            ans.inner_special = feeder.consume(1);
        }else if feeder.starts_with("*") {
            ans.text += "*";
            ans.inner_special = feeder.consume(1);
        }else if let Some(a) = ArithmeticExpr::parse(feeder, core, true) {
            ans.text += &a.text.clone();
            ans.inner = Some(a);
        }

        if ! feeder.starts_with("]") {
            return None;
        }

        ans.text += &feeder.consume(1);
        Some(ans)
    }
}
