//SPDX-FileCopyrightText: 2026 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::word::Word;
use super::BracedParamExtension;

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub text: String,
    pub offset: Option<Word>, //本来は計算式を入れられる
    pub length: Option<Word>, //同上
}

impl BracedParamExtension for Substr {
    fn get_text(&self) -> String { self.text.clone() }
    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> { Box::new(self.clone()) }
}
