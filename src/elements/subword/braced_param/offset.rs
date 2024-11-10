//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::elements::subword::BracedParam;
use crate::ShellCore;

pub fn set(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
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

    let mut ans;
    match offset.eval_as_int(core) {
        None => return false,
        Some(n) => {
            ans = obj.text.chars().enumerate()
                      .filter(|(i, _)| (*i as i64) >= n)
                      .map(|(_, c)| c).collect();
        },
    };

    if obj.has_length {
        match length(&ans, &obj.length, core) {
            Some(text) => ans = text,
            None => return false,
        }
    }

    obj.text = ans;
    true
}

fn length(text: &String, length: &Option<ArithmeticExpr>,
                         core: &mut ShellCore) -> Option<String> {
    let mut length = match length.clone() {
        None => {
            eprintln!("sush: {}: bad substitution", &text);
            return None;
        },
        Some(ofs) => ofs,
    };

    match length.eval_as_int(core) {
        None    => None,
        Some(n) => Some(text.chars().enumerate()
                        .filter(|(i, _)| (*i as i64) < n)
                        .map(|(_, c)| c).collect())
    }
}
