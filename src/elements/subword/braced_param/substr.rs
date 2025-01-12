//SPDX-FileCopyrightText: 2024 Ryuichi Ueda ryuichiueda@gmail.com
//SPDX-License-Identifier: BSD-3-Clause

use crate::elements::expr::arithmetic::ArithmeticExpr;
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

    pub fn set_partial_position_params(&mut self, array: &mut Vec<String>,
                    text: &mut String, core: &mut ShellCore) -> Result<(), String> {
        let offset = self.offset.as_mut().unwrap();
    
        if offset.text == "" {
            return Err("bad substitution".to_string());
        }
    
        *array = core.db.get_array_all("@");
        match offset.eval_as_int(core) {
            None => return Err("evaluation error".to_string()),
            Some(n) => {
                let mut start = std::cmp::max(0, n) as usize;
                start = std::cmp::min(start, array.len()) as usize;
                *array = array.split_off(start);
            },
        };
    
        if self.length.is_none() {
            *text = array.join(" ");
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
                let len = std::cmp::min(n as usize, array.len());
                let _ = array.split_off(len);
            },
        };
    
        *text = array.join(" ");
        Ok(())
    }
}
