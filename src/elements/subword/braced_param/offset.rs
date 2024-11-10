//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::BracedParam;
use crate::ShellCore;

impl BracedParam {
    pub fn offset(&mut self, core: &mut ShellCore) -> bool {
        let mut offset = match self.offset.clone() {
            None => {
                eprintln!("sush: {}: bad substitution", &self.text);
                return false;
            },
            Some(ofs) => ofs,
        };

        if offset.text == "" {
            eprintln!("sush: {}: bad substitution", &self.text);
            return false;
        }

        match offset.eval_as_int(core) {
            None => return false,
            Some(n) => {
                self.text = self.text.chars().enumerate()
                                .filter(|(i, _)| (*i as i64) >= n)
                                .map(|(_, c)| c).collect();
            },
        };

        if self.has_length {
            return self.length(core);
        }
        true
    }

    fn length(&mut self, core: &mut ShellCore) -> bool {
        let mut length = match self.length.clone() {
            None => {
                eprintln!("sush: {}: bad substitution", &self.text);
                return false;
            },
            Some(ofs) => ofs,
        };

        match length.eval_as_int(core) {
            None => false,
            Some(n) => {
                self.text = self.text.chars().enumerate()
                                .filter(|(i, _)| (*i as i64) < n)
                                .map(|(_, c)| c).collect();
                true
            },
        }
    }
}
