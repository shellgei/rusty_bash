//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::elements::subword::BracedParam;

impl BracedParam {
    pub fn replace_to_alternative(&mut self, core: &mut ShellCore) -> bool {
        let symbol = match (self.alternative_symbol.as_deref(), self.text.as_ref()) {
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

        let value: String = word.subwords.iter().map(|s| s.get_text()).collect();

        if symbol == "-" {
            self.alternative_value = None;
            self.alternative_symbol = None;
            return true;
        }
        if symbol == "+" {
            if ! core.data.has_value(&self.name) {
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
            if ! core.data.set_param(&self.name, &value) {
                return false;
            }
            self.alternative_value = None;
            self.text = value;
            return true
        }
        if symbol == ":?" {
            eprintln!("sush: {}: {}", &self.name, &value);
            return false;
        }
        if symbol == ":+" {
            self.alternative_value = match self.text.as_str() {
                "" => None,
                _  => Some(word),
            };
            return true;
        }

        return false;
    }
}
