//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

#[derive(Debug)]
pub struct Pipeline {
    pub commands: Vec<String>, //型は仮のもの
    pub pipes: Vec<String>, //同上
    pub text: String,
}
