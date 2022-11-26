//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
//use crate::feeder::scanner::*;

use crate::abst_elems::ArgElem;

pub struct SubArgStringDoubleQuoted {
    pub text: String,
    pub pos: DebugInfo,
    //pub is_value: bool,
}

impl ArgElem for SubArgStringDoubleQuoted {
    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore, as_value: bool) -> Vec<Vec<String>> {
        if as_value {
            vec!(vec!(self.text.clone()))
        }else{
            vec!(vec!(self.text.replace("\n", " ")))
        }
    }
}

impl SubArgStringDoubleQuoted {
    fn new(text: String, pos: DebugInfo/*, is_value: bool*/) -> SubArgStringDoubleQuoted {
        SubArgStringDoubleQuoted {
            text: text.clone(),
            pos: pos,
            //is_value: is_value, 
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<SubArgStringDoubleQuoted> {
        let mut pos = text.scanner_double_quoted_word();
        while pos == text.len() {
            if !text.feed_additional_line(conf){
                return None;
            }
            pos = text.scanner_double_quoted_word();
        }
        if pos == 0 {
            None
        }else{
            Some( SubArgStringDoubleQuoted::new(text.consume(pos), DebugInfo::init(text)) )
        }
    }
}
