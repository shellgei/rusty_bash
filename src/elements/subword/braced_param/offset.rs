//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::subword::BracedParam;
use crate::ShellCore;

pub fn offset(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    let mut offset = match obj.offset.clone() {
        None => {
            eprintln!("sush: {}: bad substitution", &obj.text);
            return false;
        },
        Some(ofs) => ofs,
    };

    if offset.text == "" {
        eprintln!("sush: {}: bad substitution", &obj.text);
        return false;
    }

    match offset.eval_as_int(core) {
        None => return false,
        Some(n) => {
            obj.text = obj.text.chars().enumerate()
                            .filter(|(i, _)| (*i as i64) >= n)
                            .map(|(_, c)| c).collect();
        },
    };

    if obj.has_length {
        return length(obj, core);
    }
    true
}

fn length(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    let mut length = match obj.length.clone() {
        None => {
            eprintln!("sush: {}: bad substitution", &obj.text);
            return false;
        },
        Some(ofs) => ofs,
    };

    match length.eval_as_int(core) {
        None => false,
        Some(n) => {
            obj.text = obj.text.chars().enumerate()
                            .filter(|(i, _)| (*i as i64) < n)
                            .map(|(_, c)| c).collect();
            true
        },
    }
}
