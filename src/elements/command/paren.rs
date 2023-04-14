//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder,Script};
use super::Command;
use super::super::command;

#[derive(Debug)]
pub struct ParenCommand {
    pub text: String,
    pub script: Option<Script>,
}

impl Command for ParenCommand {
    fn exec(&mut self, core: &mut ShellCore) {
        self.script.as_mut().unwrap().exec(core);//まだ仮実装
    }

    fn get_text(&self) -> String { self.text.clone() }
}

impl ParenCommand {
    fn new() -> ParenCommand {
        ParenCommand {
            text: String::new(),
            script: None,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<ParenCommand> {
        if ! feeder.starts_with("(") {
            return None;
        }

        let mut ans = Self::new();
        if ! command::parse_nested_script(feeder, core, "(", &mut ans.script, &mut ans.text){
            return None;
        }

        core.nest.pop();
        ans.text += &feeder.consume(1);
        Some(ans)
    }
}
