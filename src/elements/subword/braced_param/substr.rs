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
        let offset = self.offset.as_mut().unwrap();
    
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
            ans = self.length(&ans, core)?;
        }
    
        Ok(ans)
    }
    
    fn length(&mut self, text: &String, core: &mut ShellCore) -> Result<String, String> {
        match self.length.as_mut().unwrap().eval_as_int(core) {
            Some(n) => Ok(text.chars().enumerate()
                            .filter(|(i, _)| (*i as i64) < n)
                            .map(|(_, c)| c).collect()),
            None    => return Err("length evaluation error".to_string()),
        }
    }

    pub fn set_partial_position_params(&self, obj: &mut BracedParam, core: &mut ShellCore) -> Result<(), String> {
        let mut offset = match self.offset.clone() {
            None => {
                return Err("bad substitution".to_string());
            },
            Some(ofs) => ofs,
        };
    
        if offset.text == "" {
            return Err("bad substitution".to_string());
        }
    
        obj.array = core.db.get_array_all("@");
        match offset.eval_as_int(core) {
            None => return Err("evaluation error".to_string()),
            Some(n) => {
                let mut start = std::cmp::max(0, n) as usize;
                start = std::cmp::min(start, obj.array.len()) as usize;
                obj.array = obj.array.split_off(start);
            },
        };
    
        if self.length.is_none() {
            obj.text = obj.array.join(" ");
            return Ok(());
        }
    
        let mut length = match self.length.clone() {
            None => return Err("bad substitution".to_string()),
            Some(ofs) => ofs,
        };
    
        if length.text == "" {
            return Err("bad substitution".to_string());
        }
    
        match length.eval_as_int(core) {
            None => return Err("evaluation error".to_string()),
            Some(n) => {
                if n < 0 {
                    return Err(format!("{}: substring expression < 0", n));
                }
                let len = std::cmp::min(n as usize, obj.array.len());
                let _ = obj.array.split_off(len);
            },
        };
    
        obj.text = obj.array.join(" ");
        Ok(())
    }
}
