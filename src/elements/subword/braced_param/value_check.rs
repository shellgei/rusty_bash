//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::utils::error;
use crate::elements::subword::BracedParam;
use crate::elements::subword::braced_param::Word;

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub symbol: Option<String>,
    pub alternative_value: Option<Word>,
}

impl ValueCheck {
    pub fn set(&mut self, name: &String, text: &mut String, core: &mut ShellCore) -> bool {
        let symbol = match (self.symbol.as_deref(), text.as_ref()) {
            (Some(s), "")   => s,
            (Some("-"), _)  => "-",
            (Some(":+"), _) => ":+",
            (Some("+"), _)  => "+",
            _               => return true,
        };
    
        let word = match self.alternative_value.as_ref() {
            Some(w) => match w.tilde_and_dollar_expansion(core) {
                Some(w2) => w2,
                None     => return false,
            },
            None => return false,
        };
    
        if symbol == "-" {
            self.alternative_value = None;
            self.symbol = None;
            return true;
        }
        if symbol == "+" {
            if ! core.db.has_value(&name) {
                self.alternative_value = None;
                return true;
            }
            self.alternative_value = Some(word);
            return true;
        }
        if symbol == ":-" {
            self.alternative_value = Some(word);
            return true;
        }
        if symbol == ":=" {
            let value: String = word.subwords.iter().map(|s| s.get_text()).collect();
            if let Err(e) = core.db.set_param(&name, &value, None) {
                error::print(&e,core);
                return false;
            }
            self.alternative_value = None;
            *text = value;
            return true
        }
        if symbol == ":?" {
            let value: String = word.subwords.iter().map(|s| s.get_text()).collect();
            eprintln!("sush: {}: {}", &name, &value);
            return false;
        }
        if symbol == ":+" {
            self.alternative_value = match text.as_str() {
                "" => None,
                _  => Some(word),
            };
            return true;
        }
    
        return false;
    }

    pub fn eat(feeder: &mut Feeder, ans: &mut BracedParam, core: &mut ShellCore) -> bool {
        let num = feeder.scanner_parameter_alternative_symbol();
        if num == 0 {
            return false;
        }

        let mut info = ValueCheck::default();

        let symbol = feeder.consume(num);
        info.symbol = Some(symbol.clone());
        ans.text += &symbol;

        let num = feeder.scanner_blank(core);
        ans.text += &feeder.consume(num);
        info.alternative_value = Some(BracedParam::eat_subwords(feeder, ans, vec!["}"], core));

        ans.value_check = Some(info);
        true
    }
}
