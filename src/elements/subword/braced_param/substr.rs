//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::expr::arithmetic::ArithmeticExpr;
use crate::elements::subword::BracedParam;
use crate::ShellCore;

#[derive(Debug, Clone, Default)]
pub struct Substr {
    pub offset: Option<ArithmeticExpr>,
    pub length: Option<ArithmeticExpr>,
}

impl Substr {
    pub fn get_text(&mut self, text: &String, core: &mut ShellCore) -> Result<String, String> {
        let mut offset = self.offset.clone().unwrap();
    
        if offset.text == "" {
            return Err("bad substitution".to_string());
        }
    
        let mut ans;
        match offset.eval_as_int(core) {
            None => return Err("evaluation error".to_string()),
            Some(n) => {
                ans = text.chars().enumerate()
                          .filter(|(i, _)| (*i as i64) >= n)
                          .map(|(_, c)| c).collect();
            },
        };
    
        if self.length.is_some() {
            match self.length(&ans, core) {
                Some(text) => ans = text,
                None => return Err("length evaluation error".to_string()),
            }
        }
    
        Ok(ans)
    }
    
    fn length(&mut self, text: &String, core: &mut ShellCore) -> Option<String> {
        match self.length.as_mut()?.eval_as_int(core) {
            None    => None,
            Some(n) => Some(text.chars().enumerate()
                            .filter(|(i, _)| (*i as i64) < n)
                            .map(|(_, c)| c).collect())
        }
    }
}

pub fn set_partial_position_params(obj: &mut BracedParam, core: &mut ShellCore) -> bool {
    let info = obj.substr.clone().unwrap();

    let mut offset = match info.offset.clone() {
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

    if info.length.is_none() {
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
