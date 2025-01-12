//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::error;
use crate::elements::subword::BracedParam;
use crate::elements::subword::braced_param::Word;

#[derive(Debug, Clone, Default)]
pub struct ValueCheck {
    pub alternative_symbol: Option<String>,
    pub alternative_value: Option<Word>,
}

pub fn set(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    let check = obj.value_check.as_mut().unwrap();

    let symbol = match (check.alternative_symbol.as_deref(), obj.text.as_ref()) {
        (Some(s), "")   => s,
        (Some("-"), _)  => "-",
        (Some(":+"), _) => ":+",
        (Some("+"), _)  => "+",
        _               => return true,
    };

    let word = match check.alternative_value.as_ref() {
        Some(w) => match w.tilde_and_dollar_expansion(core) {
            Some(w2) => w2,
            None     => return false,
        },
        None => return false,
    };

    if symbol == "-" {
        check.alternative_value = None;
        check.alternative_symbol = None;
        return true;
    }
    if symbol == "+" {
        if ! core.db.has_value(&obj.param.name) {
            check.alternative_value = None;
            return true;
        }
        check.alternative_value = Some(word);
        return true;
    }
    if symbol == ":-" {
        check.alternative_value = Some(word);
        return true;
    }
    if symbol == ":=" {
        let value: String = word.subwords.iter().map(|s| s.get_text()).collect();
        if let Err(e) = core.db.set_param(&obj.param.name, &value, None) {
            error::print(&e,core);
            return false;
        }
        check.alternative_value = None;
        obj.text = value;
        return true
    }
    if symbol == ":?" {
        let value: String = word.subwords.iter().map(|s| s.get_text()).collect();
        eprintln!("sush: {}: {}", &obj.param.name, &value);
        return false;
    }
    if symbol == ":+" {
        check.alternative_value = match obj.text.as_str() {
            "" => None,
            _  => Some(word),
        };
        return true;
    }

    return false;
}
