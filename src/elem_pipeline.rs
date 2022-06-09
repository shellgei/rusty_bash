//SPDX-FileCopyrightText: 2022 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::abst_hand_input_unit::HandInputUnit;
use crate::Command;

/* command: delim arg delim arg delim arg ... eoc */
pub struct Pipeline {
    pub commands: Vec<Command>,
}

impl HandInputUnit for Pipeline {
    fn eval(&mut self, _conf: &mut ShellCore) -> Vec<String> {
        vec!()
    }

    fn exec(&mut self, _conf: &mut ShellCore) -> String{
        "".to_string()
    }
}

