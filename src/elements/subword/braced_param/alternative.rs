//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::ShellCore;
use crate::utils::error;
use crate::elements::subword::BracedParam;

pub fn set(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    let symbol = match (obj.alternative_symbol.as_deref(), obj.text.as_ref()) {
        (Some(s), "")   => s,
        (Some("-"), _)  => "-",
        (Some(":+"), _) => ":+",
        (Some("+"), _)  => "+",
        _               => return true,
    };

    let word = match obj.alternative_value.as_ref() {
        Some(w) => match w.tilde_and_dollar_expansion(core) {
            Some(w2) => w2,
            None     => return false,
        },
        None => return false,
    };

    if symbol == "-" {
        obj.alternative_value = None;
        obj.alternative_symbol = None;
        return true;
    }
    if symbol == "+" {
        if ! core.db.has_value(&obj.name.0) {
            obj.alternative_value = None;
            return true;
        }
        obj.alternative_value = Some(word);
        return true;
    }
    if symbol == ":-" {
        obj.alternative_value = Some(word);
        return true;
    }
    if symbol == ":=" {
        let value: String = word.subwords.iter().map(|s| s.get_text()).collect();
        if let Err(e) = core.db.set_param(&obj.name.0, &value, None) {
            error::print(&e,core);
            return false;
        }
        obj.alternative_value = None;
        obj.text = value;
        return true
    }
    if symbol == ":?" {
        let value: String = word.subwords.iter().map(|s| s.get_text()).collect();
        eprintln!("sush: {}: {}", &obj.name.0, &value);
        return false;
    }
    if symbol == ":+" {
        obj.alternative_value = match obj.text.as_str() {
            "" => None,
            _  => Some(word),
        };
        return true;
    }

    return false;
}
