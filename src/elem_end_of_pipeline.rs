//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::scanner::scanner_end_of_pipeline;

/* ;, \n, and comment */
#[derive(Debug)]
pub struct Eop {
    pub text: String,
    pub debug: DebugInfo,
}

impl Eop {
    //fn get_text(&self) -> String { self.text.clone() }

    pub fn parse(text: &mut Feeder) -> Option<Eop> {
        if text.len() == 0 {
            return None;
        };
    
        let pos = scanner_end_of_pipeline(text, 0);
        if pos == 0 && ! text.compare(pos, ";;") {
            return None;
        };
    
        Some(Eop{text: text.consume(pos), debug: DebugInfo::init(&text)})
    }
}
