//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::substitution::variable::Variable;
use crate::error::parse::ParseError;
use crate::{Feeder, ShellCore};
use super::Subword;

#[derive(Debug, Clone, Default)]
pub struct BracedParam {
    text: String,
    param: Variable,
}

impl Subword for BracedParam {
    fn get_text(&self) -> &str {
        self.text.as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }
}

impl BracedParam {
    fn eat_param(feeder: &mut Feeder, ans: &mut Self, core: &mut ShellCore) -> bool {
        let len = feeder.scanner_name(core);
        if len != 0 {
            ans.param = Variable::default();
            ans.param.text = feeder.consume(len);
            ans.text += &ans.param.text;
            return true;
        }

        feeder.starts_with("}")
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
                 -> Result<Option<Self>, ParseError> {
        if !feeder.starts_with("${") {
            return Ok(None);
        }
        let mut ans = Self::default();
        ans.text += &feeder.consume(2);

        if Self::eat_param(feeder, &mut ans, core) {
            ans.text += &feeder.consume(1);
        }

        Ok(Some(ans))
    }
}
