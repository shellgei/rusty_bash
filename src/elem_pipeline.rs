//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_hand_input_unit::HandInputUnit;
use crate::Command;

/* command: delim arg delim arg delim arg ... eoc */
pub struct Pipeline {
    pub commands: Vec<Command>,
    text: String,
}

impl HandInputUnit for Pipeline {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        self.commands[0].eval(conf)
    }

    fn exec(&mut self, conf: &mut ShellCore) -> String{
        self.commands[0].exec(conf)
    }
}

impl Pipeline {
    pub fn new() -> Pipeline{
        Pipeline {
            commands: vec!(),
            text: "".to_string(),
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<Pipeline> {
        if let Some(c) = Command::parse(text, conf) {
             let mut ans = Pipeline::new();
             ans.text = c.text.clone();
             ans.commands.push(c);

             Some(ans)
        }else{
            None
        }
    }
}
