//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::error::parse::ParseError;

#[derive(Debug, Clone, Default)]
pub struct Variable {
    pub text: String,
}

impl Variable {
    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_name(core);
        if len == 0 {
            return Ok(None);
        }
        let ans = Variable { text: feeder.consume(len) };
        Ok(Some(ans))
    }
}
