//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};

#[derive(Debug)]
pub struct Redirect {
    pub text: String,
    pub left_text: String,
    pub arrow: String,
    pub right_text: String,
}

impl Redirect {
    pub fn new() -> Redirect {
        Redirect {
            text: String::new(),
            left_text: String::new(),
            arrow: String::new(),
            right_text: String::new(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Redirect> {
        if feeder.starts_with("<file") || feeder.starts_with(">file") { //仮実装です。
            let mut ans = Self::new();
            ans.arrow = feeder.consume(1);
            ans.right_text = feeder.consume(4);
            ans.text = ans.arrow.clone() + &ans.right_text.clone();
            return Some(ans);
        }
        None
    }
}
