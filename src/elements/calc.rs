//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::command;
use super::word::Word;

#[derive(Debug, Clone)]
pub struct Calc {
    pub text: String,
    pub elements: Vec<String>,
}

impl Calc {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<String> {
        None
    }

    pub fn new() -> Calc {
        Calc {
            text: String::new(),
            elements: vec![],
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Calc> {
        None
    }
}
