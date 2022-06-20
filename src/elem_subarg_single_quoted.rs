//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
use crate::scanner::*;

use crate::abst_arg_elem::ArgElem;


pub struct SubArgSingleQuoted {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgSingleQuoted {
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<Vec<String>> {
        let strip = self.text[1..self.text.len()-1].to_string();
        let s = strip.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(vec!(s))
    }

    fn text(&self) -> String {
        self.text.clone()
    }
}

impl SubArgSingleQuoted {
    pub fn parse(text: &mut Feeder) -> Option<SubArgSingleQuoted> {
        if text.len() == 0 || !text.match_at(0, "'"){
            return None;
        };
    
        let pos = scanner_until(text, 1, "'");
        Some(SubArgSingleQuoted{text: text.consume(pos+1), pos: DebugInfo::init(text)})
    }
}
