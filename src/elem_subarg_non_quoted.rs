//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
use crate::scanner::*;

use crate::abst_arg_elem::ArgElem;

pub struct SubArgNonQuoted {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgNonQuoted {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> {
        vec!(self.text.clone())
    }
}

impl SubArgNonQuoted {
    pub fn parse(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        if text.len() == 0 {
            return None;
        }

        /* The first character can be { if a brace expansion is incomplete. */
        let pos = if text.nth(0) == '{' {
            scanner_until_escape(text, 1, " \n\t\"';{}()$<>&")
        }else{
            scanner_until_escape(text, 0, " \n\t\"';{}()$<>&")
        };

        if pos == 0{
            return None;
        };
        Some( SubArgNonQuoted{text: text.consume(pos), pos: DebugInfo::init(text) } )
    }

    pub fn parse2(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        let pos = scanner_until_escape(text, 0, " \n\t\"';)$<>&");
        if pos == 0{
            return None;
        };
        Some( SubArgNonQuoted{text: text.consume(pos), pos: DebugInfo::init(text) } )
    }

    pub fn parse3(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        if text.len() == 0 {
            return None;
        }
        if text.match_at(0, ",}"){
            return None;
        };
        
        let pos = scanner_until_escape(text, 0, ",{}()");
        Some( SubArgNonQuoted{ text: text.consume(pos), pos: DebugInfo::init(text) })
    }

    pub fn parse4(text: &mut Feeder) -> Option<SubArgNonQuoted> {
        if text.nth(0) == '"' {
            return None;
        };
    
        let pos = scanner_until_escape(text, 0, "\"$");
        Some( SubArgNonQuoted{text: text.consume(pos), pos: DebugInfo::init(text)})
    }
}
