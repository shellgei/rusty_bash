//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
pub struct SimpleSubword {
    pub text: String,
}

impl Subword for SimpleSubword {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn merge(&mut self, right: &Box<dyn Subword>) {
        self.text += &right.get_text().clone();
    }

    fn unquote(&mut self) {
        if ! self.text.starts_with("\\") {
            return;
        }

        self.text.remove(0);
    }
}

impl SimpleSubword {
    fn new(s: &str) -> SimpleSubword {
        SimpleSubword {
            text: s.to_string(),
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SimpleSubword> {
        let len = feeder.scanner_escaped_char(core);
        if len > 0 {
            return Some(Self::new( &feeder.consume(len) ));
        }

        let len = feeder.scanner_subword_symbol();
        if len > 0 {
            return Some(Self::new( &feeder.consume(len) ));
        }

        let len = feeder.scanner_subword();
        if len > 0 {
            return Some(Self::new( &feeder.consume(len) ));
        }

        None
    }
}
