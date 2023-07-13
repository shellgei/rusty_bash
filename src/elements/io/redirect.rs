//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};

#[derive(Debug)]
pub struct Redirect {
    pub text: String,
}

impl Redirect {
    pub fn new() -> Redirect {
        Redirect {
            text: String::new(),
        }
    }

    pub fn parse(feeder: &mut Feeder, _: &mut ShellCore) -> Option<Redirect> {
        if feeder.starts_with("<file") || feeder.starts_with(">file") { //仮実装です。
            let mut ans = Self::new();
            ans.text = feeder.consume(5);
            return Some(ans);
        }
        None
    }
}
