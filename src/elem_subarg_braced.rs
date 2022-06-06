//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ElemOfCommand;
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
        if self.args.len() == 0{
            return vec!("{}".to_string());
        }else if self.args.len() == 1{
            let mut ans = "{".to_string();
            for s in self.args[0].eval(conf) {
                ans += &s;
            };
            ans += "}";
            return vec!(ans);
        };

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
    pub fn parse(text: &mut Feeder) -> Option<SubArgBraced> {
        let pos = scanner_until(text, 0, "{");
        if pos != 0 {
            return None;
        }
        
        let mut ans = SubArgBraced {
            text: text.consume(1),
            pos: DebugInfo::init(text),
            args: vec!(),
        };
    
        while let Some(arg) = arg_in_brace(text) {
            ans.text += &arg.text.clone();
            ans.args.push(arg); 
    
            if scanner_until(text, 0, ",") == 0 {
                ans.text += &text.consume(1);
                continue;
            }else if scanner_until(text, 0, "}") == 0 {
                ans.text += &text.consume(1);
                break;
            };
        };
    
        Some(ans)
    }
}
