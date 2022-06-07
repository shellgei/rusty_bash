//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;
use crate::scanner::*;
use crate::utils::expand_tilde;

use crate::abst_arg_elem::ArgElem;

pub struct SubArgTildeUser {
    pub text: String,
    pub pos: DebugInfo,
}

impl ArgElem for SubArgTildeUser {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> {
        vec!(expand_tilde(&self.text).0)
    }
}

impl SubArgTildeUser {
    pub fn parse(text: &mut Feeder) -> Option<SubArgTildeUser> {
        if text.len() == 0 {
            return None;
        }
        if text.nth(0) != '~' {
            return None;
        }

        let pos = scanner_until_escape(text, 0, " \n\t\"';{}()$<>&*:/");
        if pos == 0{
            return None;
        };

        if text.len() > pos && !(text.nth(pos) == ':' || text.nth(pos) == '/' || text.nth(pos) == '\n') {
            return None;
        }

        Some( SubArgTildeUser{text: text.consume(pos), pos: DebugInfo::init(text) } )
    }
}
