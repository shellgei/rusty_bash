//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::elements::subword::BracedParam;
use crate::elements::subword::braced_param::Word;

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub symbol: Option<String>,
    pub alternative_value: Option<Word>,
}

impl ValueCheck {
    pub fn set(&mut self, name: &String, text: &String, core: &mut ShellCore) -> Result<String, String> {
        let mut text = text.clone();
        let symbol = match (self.symbol.as_deref(), text.as_ref()) {
            (Some(s), "")   => s,
            (Some("-"), _)  => "-",
            (Some(":+"), _) => ":+",
            (Some("+"), _)  => "+",
            _               => return Ok(text),
        };
    
        let word = match self.alternative_value.as_ref() {
            Some(w) => match w.tilde_and_dollar_expansion(core) {
                Some(w2) => w2,
                None     => return Err("expansion error".to_string()),
            },
            None => return Err("no alternative value".to_string()),
        };
    
        if symbol == "-" {
            self.alternative_value = None;
            self.symbol = None;
            return Ok(text);
        }
        if symbol == "+" {
            if ! core.db.has_value(&name) {
                self.alternative_value = None;
                return Ok(text);
            }
            self.alternative_value = Some(word);
            return Ok(text);
        }
        if symbol == ":-" {
            self.alternative_value = Some(word);
            return Ok(text);
        }
        if symbol == ":=" {
            let value: String = word.subwords.iter().map(|s| s.get_text()).collect();
            core.db.set_param(&name, &value, None)?;

            self.alternative_value = None;
            text = value;
            return Ok(text);
        }
        if symbol == ":?" {
            let value: String = word.subwords.iter().map(|s| s.get_text()).collect();
            eprintln!("sush: {}: {}", &name, &value);
            return Err("".to_string());
        }
        if symbol == ":+" {
            self.alternative_value = match text.as_str() {
                "" => None,
                _  => Some(word),
            };
            return Ok(text);
        }
    
        return Err("no operation".to_string());
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
