//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::elements::subword::BracedParam;
use crate::ShellCore;

pub fn set(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    let info = obj.offset.clone().unwrap();

    let mut offset = match obj.offset.clone() {
        None => {
            eprintln!("sush: {}: bad substitution", &obj.text);
            return false;
        },
        Some(ofs) => ofs,
    };

    if info.offset.clone().unwrap().text == "" {
        eprintln!("sush: {}: bad substitution", &obj.text);
        return false;
    }

    let mut ans;
    match info.offset.unwrap().eval_as_int(core) {
        None => return false,
        Some(n) => {
            ans = obj.text.chars().enumerate()
                      .filter(|(i, _)| (*i as i64) >= n)
                      .map(|(_, c)| c).collect();
        },
    };

    if info.has_length {
        match length(&ans, &info.length, core) {
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

pub fn set_partial_position_params(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    let info = obj.offset.clone().unwrap();

    let mut offset = match obj.offset.clone().unwrap().offset.clone() {
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

    obj.array = core.db.get_array_all("@");
    match offset.eval_as_int(core) {
        None => return false,
        Some(n) => {
            let mut start = std::cmp::max(0, n) as usize;
            start = std::cmp::min(start, obj.array.len()) as usize;
            obj.array = obj.array.split_off(start);
        },
    };

    if ! info.has_length {
        obj.text = obj.array.join(" ");
        return true;
    }

    let mut length = match info.length.clone() {
        None => {
            eprintln!("sush: {}: bad substitution", &obj.text);
            return false;
        },
        Some(ofs) => ofs,
    };

    if length.text == "" {
        eprintln!("sush: {}: bad substitution", &obj.text);
        return false;
    }

    match length.eval_as_int(core) {
        None => return false,
        Some(n) => {
            if n < 0 {
                eprintln!("{}: substring expression < 0", n);
                return false;
            }
            let len = std::cmp::min(n as usize, obj.array.len());
            let _ = obj.array.split_off(len);
        },
    };

    obj.text = obj.array.join(" ");
    true
}
