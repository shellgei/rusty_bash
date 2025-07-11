//SPDX-FileCopyrightText: 2025 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;

#[derive(Debug, Clone, Default)]
pub struct Value {
    pub text: String,
    pub value: Word,
    pub evaluated_string: String,
}
