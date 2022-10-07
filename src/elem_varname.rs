//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::debuginfo::DebugInfo;
use crate::ShellCore;
use crate::Feeder;

use crate::abst_elems::ArgElem;

pub struct VarName {
    pub text: String,
    pub pos: DebugInfo,
}

impl VarName {
    pub fn new(text: &mut Feeder, length: usize) -> VarName{
        VarName{
            text: text.consume(length),
            pos: DebugInfo::init(text),
        }
    }
}

impl ArgElem for VarName {
    fn get_text(&self) -> String {
        self.text.clone()
    }

    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<Vec<String>> {
        vec![]
    }
}

