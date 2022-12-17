//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
//use crate::feeder::scanner::*;

use crate::element::subword::Subword;


pub struct SubwordSingleQuoted {
    pub text: String,
    pub pos: DebugInfo,
    //pub is_value: bool,
}

impl Subword for SubwordSingleQuoted {
    fn eval(&mut self, _conf: &mut ShellCore, _: bool) -> Vec<Vec<String>> {
        let strip = self.text[1..self.text.len()-1].to_string();
        let s = strip.replace("\\", "\\\\").replace("*", "\\*"); 
        vec!(vec!(s))
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl SubwordSingleQuoted {
    pub fn parse(text: &mut Feeder, core: &mut ShellCore) -> Option<SubwordSingleQuoted> {
        if ! text.starts_with("'") {
            return None;
        };
    
        let mut pos = text.scanner_until(1, "'");
        while pos == text.len() {
            if !text.feed_additional_line(core){
                return None;
            }
            pos = text.scanner_until(1, "'");
        }
        Some(SubwordSingleQuoted{text: text.consume(pos+1),
                                pos: DebugInfo::init(text)})
    }
}
