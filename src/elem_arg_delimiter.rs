//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::Feeder;
use crate::scanner::scanner_while;
use crate::abst_command_elem::CommandElem;

/* delimiter */
#[derive(Debug)]
pub struct ArgDelimiter {
    pub text: String,
    pub debug: DebugInfo,
}

impl CommandElem for ArgDelimiter {
    fn parse_info(&self) -> Vec<String> {
        vec!(format!("    delimiter: '{}' ({})", self.text.clone(), self.debug.text()))
    }

    fn text(&self) -> String { self.text.clone() }
}

impl ArgDelimiter{
    pub fn return_if_valid(text: &mut Feeder, pos: usize) -> Option<ArgDelimiter> {
        if pos == 0 {
            return None;
        };

        Some(ArgDelimiter{text: text.consume(pos), debug: DebugInfo::init(text)})
    }

    pub fn parse(text: &mut Feeder) -> Option<ArgDelimiter> {
        let pos = scanner_while(text, 0, " \t");
        ArgDelimiter::return_if_valid(text, pos)
    }
}
