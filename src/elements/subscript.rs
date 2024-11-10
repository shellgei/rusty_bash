//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};

#[derive(Debug, Clone, Default)]
pub struct Subscript {
    pub text: String,
}

impl Subscript {
    pub fn eval(&mut self) -> Option<String> {
        let len = self.text.len();
        let inner = &self.text[1..len-1];

        if inner.len() == 1 {
            if let Some(ch) = inner.chars().nth(0) {
                if '0' <= ch && ch <= '9' || ch == '@' {
                    return Some(inner.to_string());
                }
            }
        }

        None
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Self> {
        if ! feeder.starts_with("[") {
            return None;
        }

        let mut ans = Self::default();
        ans.text += &feeder.consume(1);

        while ! feeder.starts_with("]") {
            let len = feeder.scanner_inner_subscript(core);
            ans.text += &feeder.consume(len);
        }

        ans.text += &feeder.consume(1);
        Some(ans)
    }
}
