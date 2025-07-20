//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::error::exec::ExecError;
use super::super::Variable;
use super::OptionalOperation;

impl OptionalOperation for Escape {
    fn get_text(&self) -> String {self.text.clone()}
    fn exec(&mut self, _: &Variable, text: &String, _: &mut ShellCore) -> Result<String, ExecError> {
        match self.symbol.as_ref() {
            "k" | "K" | "Q" => {
                let text = format!("'{}'", &text);
                return Ok(text);
            },
            _ => {},
        }
        Ok(text.clone())
    }

    fn boxed_clone(&self) -> Box<dyn OptionalOperation> {Box::new(self.clone())}
}

#[derive(Debug, Clone, Default)]
pub struct Escape {
    pub text: String,
    pub symbol: String,
}

impl Escape {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("@") {
            return None;
        }

        let mut ans = Escape::default();
        if feeder.scanner_escape_directive_in_braced_param(core) == 2 {
            ans.text = feeder.consume(1);
            ans.symbol = feeder.consume(1);
            ans.text += &ans.symbol;
        }else{
            return None;
        }

        Some(ans)
    }
}
