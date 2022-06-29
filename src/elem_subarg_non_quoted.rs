//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
use crate::scanner::*;

use crate::abst_elems::ArgElem;

pub struct SubArgNonQuoted {
    pub text: String,
    pub pos: DebugInfo,
    pub is_value: bool,
}

impl ArgElem for SubArgNonQuoted {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<Vec<String>> {
        if self.is_value {
            vec!(vec!(self.text.clone()))
        }else{
            vec!(vec!(self.text.replace("\n", " ")))
        }
    }
}

impl SubArgNonQuoted {
    fn new(text: String, pos: DebugInfo, is_value: bool) -> SubArgNonQuoted {
        SubArgNonQuoted {
            text: text.clone(),
            pos: pos,
            is_value: is_value, 
        }
    }

    pub fn parse(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        if text.len() == 0 {
            return None;
        }

        let pos = scanner_until_escape(text, 0, " \n\t\"';{()$<>&");
        if pos == 0{
            None
        }else{
            Some( SubArgNonQuoted::new(text.consume(pos), DebugInfo::init(text), false) )
        }
    }

    pub fn parse3(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        if text.len() == 0 {
            return None;
        }

        
        if text.nth_is(0, ",}"){
            return None;
        };
        
        let pos = scanner_until_escape(text, 0, ",{}()");
        let backup = text.clone();

        let ans = Some( SubArgNonQuoted::new(text.consume(pos), DebugInfo::init(text), false) );
        if text.len() == 0 || scanner_end_of_com(text, 0) == 1 {
            text.rewind(backup);
            return None;
        }

        ans
    }

    pub fn parse_in_dq(text: &mut Feeder, conf: &mut ShellCore, is_value: bool) -> Option<SubArgNonQuoted> {
        if text.len() == 0 {
            if !text.feed_additional_line(conf){
                return None;
            }
        }
    
        let mut pos = scanner_until_escape(text, 0, "\"$");
        while pos == text.len() {
            if !text.feed_additional_line(conf){
                return None;
            }
            pos = scanner_until_escape(text, 0, "\"$");
        }

        Some( SubArgNonQuoted::new(text.consume(pos), DebugInfo::init(text), is_value) )
        //Some( SubArgNonQuoted{text: text.consume(pos), pos: DebugInfo::init(text)})
    }
}
