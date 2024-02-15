//SPDX-FileCopyrightText: 2024 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::Subword;

#[derive(Debug, Clone)] //Cloneも指定しておく
pub struct Word {
    pub text: String,
    pub subwords: Vec<Box<dyn Subword>>,
}

impl Word {
    pub fn eval(&mut self) -> Vec<String> {
        vec![self.text.clone()]
    }
}
