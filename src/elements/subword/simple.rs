//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{ShellCore, Feeder};
use crate::elements::subword::Subword;

#[derive(Debug, Clone)]
enum SubwordType {
    /* parameters and variables */
    ParamSpecialPositional,
    VarName,
    /* simple subwords */
    SingleQuoted,
    Symbol,
    Escaped,
    Other,
}

#[derive(Debug, Clone)]
pub struct SimpleSubword {
    pub text: String,
    subword_type: SubwordType,
}

impl Subword for SimpleSubword {
    fn get_text(&self) -> &str {&self.text.as_ref()}
    fn boxed_clone(&self) -> Box<dyn Subword> {Box::new(self.clone())}

    fn merge(&mut self, right: &Box<dyn Subword>) {
        self.text += &right.get_text();
    }

    fn parameter_expansion(&mut self, core: &mut ShellCore) {
        match self.subword_type {
            SubwordType::ParamSpecialPositional => {
                let value = core.get_param_ref(&self.text[1..]);
                self.text = value.to_string();
            },
            _ => {},
        }
    }

    fn unquote(&mut self) {
        match self.subword_type {
            SubwordType::SingleQuoted => {
                self.text.remove(0);
                self.text.pop();
            },
            SubwordType::Escaped => {
                self.text.remove(0);
            },
            _ => {},
        }
    }
}

impl SimpleSubword {
    fn new(s: &str, tp: SubwordType) -> SimpleSubword {
        SimpleSubword {
            text: s.to_string(),
            subword_type: tp,
        }
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore) -> Option<SimpleSubword> {
        let len = feeder.scanner_dollar_special_and_positional_param(core);
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::ParamSpecialPositional));
        }

        let len = feeder.scanner_name(core);
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::VarName));
        }

        let len = feeder.scanner_single_quoted_subword(core);
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::SingleQuoted));
        }
    
        let len = feeder.scanner_escaped_char(core);
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::Escaped));
        }

        let len = feeder.scanner_subword_symbol();
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::Symbol));
        }

        let len = feeder.scanner_subword();
        if len > 0 {
            return Some(Self::new(&feeder.consume(len), SubwordType::Other));
        }

        None
    }
}
