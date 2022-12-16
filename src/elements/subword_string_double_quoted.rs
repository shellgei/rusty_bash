//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
//use crate::feeder::scanner::*;

use crate::elements::abst_subword::WordElem;

pub struct SubWordStringDoubleQuoted {
    pub text: String,
    pub pos: DebugInfo,
    //pub is_value: bool,
}

impl WordElem for SubWordStringDoubleQuoted {
    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore, remove_lf: bool) -> Vec<Vec<String>> {
        if remove_lf {
            vec!(vec!(self.text.replace("\n", " ")))
        }else{
            vec!(vec!(self.text.clone()))
        }
    }
}

impl SubWordStringDoubleQuoted {
    fn new(text: String, pos: DebugInfo/*, is_value: bool*/) -> SubWordStringDoubleQuoted {
        SubWordStringDoubleQuoted {
            text: text.clone(),
            pos: pos,
            //is_value: is_value, 
        }
    }

    pub fn parse(text: &mut Feeder, conf: &mut ShellCore) -> Option<SubWordStringDoubleQuoted> {
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
            Some( SubWordStringDoubleQuoted::new(text.consume(pos), DebugInfo::init(text)) )
        }
    }
}
