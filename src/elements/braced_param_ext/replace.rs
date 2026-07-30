//SPDX-FileCopyrightText: 2026 Ryuichi Ueda <ryuichiueda@gmail.com>
//SPDX-License-Identifier: BSD-3-Clause

use crate::{Feeder, ShellCore};
use crate::utils::glob;
use crate::utils::glob::GlobElem;
use crate::elements::word::{Word, WordMode};
use super::{BracedParamExtension, ExecError, ParseError, Parameter};

#[derive(Debug, Clone, Default)]
pub struct Replace {
    pub text: String,
    pub symbol: String,
    pub pattern: Word,
    pub string: Word,
}

impl BracedParamExtension for Replace {
    fn exec(&mut self, v: &Parameter, text: &str, core: &mut ShellCore)
        -> Result<String, ExecError> {
        if ! core.db.exist(&v.text) {
            return Ok("".to_string());
        }

        let mut text = text.to_string();
        let pat_str = self.pattern.eval_as_pattern(core)?;
        let pat = glob::parse(&pat_str);

        if self.symbol.starts_with("#") {
            let len = glob::match_length(&text, &pat, self.symbol == "##");
            text = text[len..].to_string();
        } else if self.symbol.starts_with("%") {
            self.percent(&mut text, &pat);
        }

        Ok(text)
    }

    fn boxed_clone(&self) -> Box<dyn BracedParamExtension> {
        Box::new(self.clone())
    }

    fn get_text(&self) -> String {
        self.text.clone()
    }
}

impl Replace {
    pub fn percent(&self, text: &mut String, pattern: &[GlobElem]) {
        let mut length = text.len();
        let mut ans_length = length; //ans_length: 最終的に残す文字列の長さ
     
        for ch in text.chars().rev() { //文字列の末端から走査
            length -= ch.len_utf8();   //部分文字列の開始位置を計算
            let s = text[length..].to_string();
     
            if glob::compare(&s, pattern) {
                ans_length = length;
                if self.symbol == "%" {
                    break; //最短一致の場合はここで終わり
                }   
            }   
        }   
     
        *text = text[0..ans_length].to_string(); //マッチした部分を削除
    }

    pub fn parse(feeder: &mut Feeder, core: &mut ShellCore)
    -> Result<Option<Self>, ParseError> {
        let len = feeder.scanner_parameter_remove_symbol();
        if len == 0 { 
            return Ok(None);
        }   

        let mut ans = Replace::default();
        ans.symbol = feeder.consume(len);
        ans.text += &ans.symbol.clone();

        let mode = Some(WordMode::PermitAnyUntil(vec!["}".to_string()]));
        ans.pattern = Word::parse(feeder, core, mode)?.unwrap_or_default();
        ans.text += &ans.pattern.text.clone();
//        dbg!("{:?}", &ans);
        Ok(Some(ans))
    }
}
