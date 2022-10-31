//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::scanner::scanner_end_of_com;

/* ;, \n, and comment */
#[derive(Debug)]
pub struct Eoc {
    pub text: String,
    pub debug: DebugInfo,
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
