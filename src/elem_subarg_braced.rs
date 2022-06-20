//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::abst_command_elem::CommandElem;
use crate::ShellCore;
use crate::Feeder;
use crate::elem_arg::Arg;
use crate::scanner::*;

use crate::elem_arg::arg_in_brace;
use crate::abst_arg_elem::ArgElem;

pub struct SubArgBraced {
    pub text: String,
    pub pos: DebugInfo,
    pub args: Vec<Arg>
}

impl ArgElem for SubArgBraced {
    fn eval(&mut self, conf: &mut ShellCore) -> Vec<String> {
        let mut ans = vec!();
        for arg in &mut self.args {
            ans.append(&mut arg.eval(conf));
        };
        ans
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgBraced {
    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<SubArgBraced> {
        if text.len() == 0 {
            return None;
        }

        let pos = scanner_until(text, 0, "{");
        if pos != 0 {
            return None;
        }
        
        let backup = text.clone();
        let mut ans = SubArgBraced {
            text: text.consume(1),
            pos: DebugInfo::init(text),
            args: vec!(),
        };
    
        while let Some(arg) = arg_in_brace(text, conf) {
            ans.text += &arg.text.clone();
            ans.args.push(arg); 

            if text.len() == 0 {
                text.rewind(backup);
                return None;
            }
    
            if scanner_until(text, 0, ",") == 0 {
                ans.text += &text.consume(1);
                continue;
            }else if scanner_until(text, 0, "}") == 0 {
                ans.text += &text.consume(1);
                break;
            };
        };

        if ans.args.len() < 2 {
            text.rewind(backup);
            return None;
        }
    
        Some(ans)
    }
}
