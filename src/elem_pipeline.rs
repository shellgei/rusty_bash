//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::abst_hand_input_unit::HandInputUnit;
use crate::Command;
use crate::elem_arg_delimiter::ArgDelimiter;

/* command: delim arg delim arg delim arg ... eoc */
pub struct Pipeline {
    pub commands: Vec<Command>,
    text: String,
}

impl HandInputUnit for Pipeline {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let x = self.commands.len();
        if x == 0 {
            return vec!();
        }
        self.commands[x-1].eval(conf)
    }

    fn exec(&mut self, conf: &mut ShellCore) -> String{
        let x = self.commands.len();
        eprintln!("{}", x);
        if x == 0 {
            return "".to_string();
        }
        self.commands[x-1].exec(conf)
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
        let mut ans = Pipeline::new();

        loop {
            if let Some(c) = Command::parse(text, conf) {
                let mut eoc = "".to_string();
                if let Some(e) = c.args.last() {
                    eoc = e.text();
                }

                ans.text += &c.text.clone();
                ans.commands.push(c);

                if eoc != "|" {
                    break;
                }

                if let Some(d) = ArgDelimiter::parse(text) {
                    ans.text += &d.text.clone();
                }

                /*

                if text.len() == 0{
                    break;
                }

                if text.nth(0) != '|' {
                    break;
                }
                */

            }else{
                break;
            }
        }

        if ans.commands.len() > 0 {
            Some(ans)
        }else{
            None
        }
    }
}
