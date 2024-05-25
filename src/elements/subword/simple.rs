//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::Feeder;
use crate::elements::subword::{Subword, SubwordType};

#[derive(Debug, Clone)]
pub struct SimpleSubword {
    pub text: String,
}

impl Subword for SimpleSubword {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn set_text(&mut self, text: &str) { self.text = text.to_string(); }
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}
    fn get_type(&self) -> SubwordType { SubwordType::Simple }
}

impl SimpleSubword {
    pub fn parse(feeder: &mut Feeder) -> Option<SimpleSubword> {
        let len = feeder.scanner_subword_symbol();
        if len > 0 {
            return Some( Self{ text :feeder.consume(len) } );
        }

        let len = feeder.scanner_subword();
        if len > 0 {
            return Some( Self{ text :feeder.consume(len) } );
        }

        None
    }
}
