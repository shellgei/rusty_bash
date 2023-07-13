//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::io;
use std::os::unix::prelude::RawFd;
use nix::unistd;

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
        if feeder.starts_with("<") || feeder.starts_with(">") { //仮実装です。
            let mut ans = Redirect::new();
            ans.arrow = feeder.consume(1);
            ans.text = ans.arrow.clone();
            return Some(ans);
        }
        None
    }
}
