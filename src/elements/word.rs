//SPDX-FileCopyrightText: 2023 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug)]
pub struct Word {
    pub text: String,
}

impl Word {
    pub fn get_args(&mut self) -> Vec<String> {
        vec![self.text.clone()]
    }
}
