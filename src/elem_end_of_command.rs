//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::scanner::scanner_end_of_com;
use crate::abst_elems::CommandElem;

/* ;, \n, and comment */
#[derive(Debug)]
pub struct Eoc {
    pub text: String,
    pub debug: DebugInfo,
}

impl CommandElem for Eoc {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    end mark : '{}' ({})\n", self.text.clone(), self.debug.text()))
    }

    fn text(&self) -> String { self.text.clone() }
}

impl Eoc {
    pub fn parse(text: &mut Feeder) -> Option<Eoc> {
        if text.len() == 0 {
            return None;
        };
    
        let pos = scanner_end_of_com(text, 0);
        if pos == 0 {
            return None;
        };
    
        Some(Eoc{text: text.consume(pos), debug: DebugInfo::init(&text)})
    }
}
