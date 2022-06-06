//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;

#[derive(Debug)]
pub struct DebugInfo {
    pub lineno: u32,
    pub pos: u32,
    pub comment: String,
}

impl DebugInfo {
    pub fn text(&self) -> String {
        format!("lineno: {}, pos: {} {}", 
                self.lineno.to_string(),
                self.pos.to_string(),
                self.comment)
    }

    pub fn init(f: &Feeder) -> DebugInfo {
        DebugInfo {
            lineno: f.lineno().0,
            pos: f.pos(),
            comment: "".to_string()
        }
    }
}

