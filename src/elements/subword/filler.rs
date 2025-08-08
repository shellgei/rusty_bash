//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use super::Subword;

/* This subword disappers at word split. */
#[derive(Debug, Clone)]
pub struct FillerSubword {
    pub text: String,
}

impl Subword for FillerSubword {
    fn get_text(&self) -> &str {
        &self.text.as_ref()
    }
    fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
    fn boxed_clone(&self) -> Box<dyn Subword> {
        Box::new(self.clone())
    }
}
