//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore,Feeder,Script};
use super::Command;

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

    fn eat_script(feeder: &mut Feeder, core: &mut ShellCore, ans: &mut ParenCommand) -> bool {
        if let Some(s) = Script::parse(feeder, core) {
            ans.text += &s.text;
            ans.script = Some(s);
            return true;
        }
        false
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<ParenCommand> {
        if ! feeder.starts_with("(") {
            return None;
        }
        core.nest.push("(".to_string());

        let mut ans = Self::new();
        let backup = feeder.clone();

        ans.text = feeder.consume(1);
        if ! Self::eat_script(feeder, core, &mut ans){
            feeder.rewind(backup);
            core.nest.pop();
            return None;
        }
        // ans.text += &feeder.consume(1); あとからコメントアウトを外す

        core.nest.pop();
        Some(ans)
    }
}
