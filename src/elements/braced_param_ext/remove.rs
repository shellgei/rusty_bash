//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;

#[derive(Debug, Clone, Default)]
pub struct Remove {
    pub text: String,
    pub symbol: String,
    pub pattern: Option<Word>,
}
