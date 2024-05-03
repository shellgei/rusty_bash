//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use super::subword;
use super::subword::Subword;
use super::word::Word;

#[derive(Debug, Clone)]
pub struct Array {
    pub text: String,
    pub words: Vec<Word>,
}

impl Array {
    pub fn eval(&mut self, core: &mut ShellCore) -> Option<Vec<String>> {
        None
    }

    pub fn new() -> Array {
        Array {
            text: String::new(),
            words: vec![],
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<Array> {
        None
    }
}
