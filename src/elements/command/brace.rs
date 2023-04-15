//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder,Script};
use super::Command;

#[derive(Debug)]
pub struct BraceCommand {
    pub text: String,
    pub script: Option<Script>,
}

impl Command for BraceCommand {
    fn exec(&mut self, core: &mut ShellCore) {
        self.script.as_mut().unwrap().exec(core);//まだ仮実装
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl BraceCommand {
    fn new() -> BraceCommand {
        BraceCommand {
            text: String::new(),
            script: None,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<BraceCommand> {
        match Script::parse_with_left(feeder, core, "{") {
            Some(s) => {
                let mut ans = Self::new();
                ans.text = "{".to_string() + &s.text.clone() + &feeder.consume(1);
                ans.script = Some(s);
                Some(ans)
            },
            None => None, 
        }
    }
}
