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
        if let Some(s) = self.script.as_mut() {
            s.exec(core);
        }
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
       if let Some(script) = Script::parse(feeder, core){
           ans.text += &script.text.clone();
           ans.script = Some(script);
           true
       }else{
           false
       }
   }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<ParenCommand> {
        if ! feeder.starts_with("("){
            return None;
        }

        let mut ans = Self::new();
        let backup = feeder.clone();

        ans.text += &feeder.consume(1);

        if ! Self::eat_script(feeder, core, &mut ans){
            feeder.rewind(backup);
            return None;
        }

        if ! feeder.starts_with(")"){
            feeder.rewind(backup);
            None
        }else{
            ans.text += &feeder.consume(1);
            let blank_len = feeder.scanner_blank();
            ans.text += &feeder.consume(blank_len);
            Some(ans)
        }
    }
}
